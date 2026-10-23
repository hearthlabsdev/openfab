use okapi::map;
use okapi::openapi3::Responses;
use okapi::openapi3::SecurityScheme;
use okapi::openapi3::SecuritySchemeData;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome};
use rocket::{Request, State};
use rocket_okapi::OpenApiError;
use rocket_okapi::r#gen::OpenApiGenerator;
use rocket_okapi::okapi::schemars;
use rocket_okapi::okapi::schemars::JsonSchema;
use rocket_okapi::request::{OpenApiFromRequest, RequestHeaderInput};
use rocket_okapi::util::add_schema_response;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serverlib::accounts::User;
use serverlib::admin::apikeys::ApiKeyORM;
use uuid::Uuid;

pub struct ApiGuard {
    pub api_key: ApiKeyORM,
}

impl ApiGuard {
    pub fn owner(&self) -> Uuid {
        self.api_key.owner.id
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiGuard {
    type Error = ErrorResponse;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let pool = match request.rocket().state::<ormlite::postgres::PgPool>() {
            Some(pool) => pool,
            None => {
                return Outcome::Error((
                    Status::InternalServerError,
                    ErrorResponse {
                        message: "Internal Server Error".into(),
                        code: 500,
                    },
                ));
            }
        };
        let mut conn = match pool.acquire().await {
            Ok(conn) => conn,
            Err(_) => {
                return Outcome::Error((
                    Status::InternalServerError,
                    ErrorResponse {
                        message: "Internal Server Error".into(),
                        code: 500,
                    },
                ));
            }
        };

        let authorization = match request.headers().get_one("Authorization") {
            Some(key) => key.strip_prefix("Bearer ").unwrap_or(key),
            None => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "Authorization header not found headers: {:?}",
                    request.headers()
                );
                return Outcome::Forward(Status::Unauthorized);
            }
        };

        let key = match ApiKeyORM::get_by_key(pool, authorization).await {
            Ok(key) => key,
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("Invalid API key: {}", e);
                return Outcome::Forward(Status::Unauthorized);
            }
        };
        Outcome::Success(ApiGuard { api_key: key })
    }
}

#[derive(Debug, Serialize, Clone, Deserialize, JsonSchema)]
pub struct ErrorResponse {
    pub message: String,
    pub code: u16,
}

impl<'a> OpenApiFromRequest<'a> for ApiGuard {
    // Required method
    fn from_request_input(
        _generator: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> Result<RequestHeaderInput, OpenApiError> {
        Ok(RequestHeaderInput::Security(
            "Bearer".into(),
            SecurityScheme {
                description: Some("API Key for authentication".into()),
                data: SecuritySchemeData::Http {
                    scheme: "Bearer".into(),
                    bearer_format: Some("JWT".into()),
                },
                extensions: map![],
            },
            map!["Bearer".into() => vec![]],
        ))
    }

    // Provided method
    fn get_responses(generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Responses::default();
        let schema = generator.json_schema::<ErrorResponse>();
        add_schema_response(&mut responses, 403, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 401, "application/json", schema.clone())?;
        add_schema_response(&mut responses, 500, "application/json", schema.clone())?;
        Ok(responses)
    }
}
