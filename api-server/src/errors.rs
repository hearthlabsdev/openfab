use okapi::openapi3::Responses;
use rocket::{
    Request, Response,
    http::Status,
    response::{Redirect, Responder},
};
use rocket_okapi::util::add_schema_response;
use rocket_okapi::{JsonSchema, OpenApiError};
use rocket_okapi::{r#gen::OpenApiGenerator, response::OpenApiResponderInner};
use serde::{Deserialize, Serialize};
use serverlib::errors::{OpenFabErr, AuthErr};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, JsonSchema, Error)]
#[error("OpenFab error: {message}")]
pub struct ErrorResponse {
    pub message: String,
    pub status: u16,
}

impl ErrorResponse {
    pub fn new(message: String, status: u16) -> Self {
        ErrorResponse { message, status }
    }

    pub fn unimplemented(route: &str) -> Self {
        ErrorResponse {
            message: format!("Route not implemented: {}", route),
            status: 501,
        }
    }
}

impl From<OpenFabErr> for ErrorResponse {
    fn from(err: OpenFabErr) -> Self {
        let (status, message) = err.match_status_message();
        ErrorResponse {
            message,
            status: status.code,
        }
    }
}

impl From<ormlite::Error> for ErrorResponse {
    fn from(err: ormlite::Error) -> Self {
        ErrorResponse {
            message: err.to_string(),
            status: 500,
        }
    }
}

impl From<AuthErr> for ErrorResponse {
    fn from(err: AuthErr) -> Self {
        ErrorResponse {
            message: err.to_string(),
            status: 403,
        }
    }
}

impl From<ormlite::SqlxError> for ErrorResponse {
    fn from(err: ormlite::SqlxError) -> Self {
        ErrorResponse {
            message: err.to_string(),
            status: 500,
        }
    }
}

impl OpenApiResponderInner for ErrorResponse {
    fn responses(generator: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Responses::default();
        let schema = generator.json_schema::<ErrorResponse>();
        add_schema_response(&mut responses, 500, "application/json", schema.clone());
        add_schema_response(&mut responses, 501, "application/json", schema.clone());
        add_schema_response(&mut responses, 400, "application/json", schema.clone());
        add_schema_response(&mut responses, 401, "application/json", schema.clone());
        add_schema_response(&mut responses, 403, "application/json", schema.clone());
        add_schema_response(&mut responses, 404, "application/json", schema.clone());
        add_schema_response(&mut responses, 422, "application/json", schema.clone());
        add_schema_response(&mut responses, 303, "application/json", schema.clone());
        Ok(responses)
    }
}

impl<'r> Responder<'r, 'static> for ErrorResponse {
    fn respond_to(self, _request: &Request) -> rocket::response::Result<'static> {
        let status = Status::from_code(self.status).unwrap_or(Status::InternalServerError);
        let response = serde_json::to_string(&self).expect("failed to serialize message");
        Response::build()
            .status(status)
            .sized_body(response.len(), std::io::Cursor::new(response))
            .ok()
    }
}
/*
impl OpenApiResponderInner for ErrorResponse {
    fn responses(gen: &mut OpenApiGenerator) -> Result<Responses, OpenApiError> {
        let mut responses = Responses::default();
        let schema = gen.json_schema::<ErrorResponse>()?;
        let response = Response {
            description: "Error response".to_string(),
            content: Some(Content::from_json(schema)),
            ..Default::default()
        };
        responses.responses.insert("default".to_string(), response);
        Ok(responses)
    }
}*/
