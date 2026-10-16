use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rocket::{Request, Response, http::Status, response::Redirect, response::Responder};
use thiserror::Error;

/// general purpose error
#[derive(Debug, Error)]
pub enum OpenFabErr {
    /// database error, usually involving the ORM
    #[error("database error: {0}")]
    OrmLite(#[from] ormlite::Error),

    /// underlying database error usually involving generated queries.
    #[error("sql error: {0}")]
    SQLXErr(#[from] ormlite::SqlxError),

    /// object store (S3 Client) errors.
    #[error("Minio error: {0}")]
    MinioErr(#[from] minio_rsc::error::Error),

    /// I don't honestly know why this is a separate error, I haven't checked
    #[error("Minio value error: {0}")]
    ValueErr(#[from] minio_rsc::error::ValueError),

    /// failed to load secret required for running the application.
    #[error("secret fetch error: {0}")]
    SecretFetchErr(#[from] secret_ref::SecretError),

    /// failed to authenticate a user
    #[error("authentication failed: {0}")]
    AuthErr(#[from] AuthErr),

    /// 3D Printer Driver Error
    #[error("driver error: {0}")]
    DriverError(#[from] openfab_drivers::errors::DriverError),

    /// Driver Configuration Failure
    #[error("driver config error: {0}")]
    DriverConfigError(#[from] openfab_drivers::errors::ConfigError),

    /// failed to parse a UUID from a string.
    #[error("failed to parse string to uuid: {0}")]
    UuidParseErr(#[from] uuid::Error),

    /// a driver  or device required a configuraiton opton that wasn't found.
    #[error("failed to parse config missing: {1} from {0}")]
    MissingConfigOption(String, String),

    /// JSON Serialization or Deserializaton error
    #[error("json serialization error: {0}")]
    JsonErr(#[from] serde_json::Error),
}

impl OpenFabErr {
    pub(crate) fn match_status_message(&self) -> (Status, String) {
        let (status, message) = match &self {
            OpenFabErr::AuthErr(auth_err) => {
                // Delegate to AuthErr's status mapping
                match auth_err {
                    AuthErr::InvalidPassword => (Status::Unauthorized, "invalid password".into()),
                    AuthErr::UserNotFound => (Status::Unauthorized, "user not found".into()),
                }
            }
            OpenFabErr::UuidParseErr(_) => (Status::BadRequest, "failed to parse uuid".into()),
            OpenFabErr::SecretFetchErr(_) => {
                (Status::InternalServerError, "internal server error".into())
            }
            OpenFabErr::MissingConfigOption(route, option) => (
                Status::BadRequest,
                format!("missing config option {option} for {route}"),
            ),
            // Map remaining database, driver, or external service errors to InternalServerError (or BadGateway)
            val => (Status::InternalServerError, val.to_string()),
        };

        (status, message)
    }
}

impl<'r> Responder<'r, 'static> for OpenFabErr {
    fn respond_to(self, req: &Request<'_>) -> rocket::response::Result<'static> {
        match &self {
            // If it's an authentication error, redirect back to the login page with the error message
            OpenFabErr::AuthErr(_auth_err) => {
                let (_status, error_msg) = self.match_status_message();
                let encoded_msg = utf8_percent_encode(&error_msg, NON_ALPHANUMERIC).to_string();

                let encoded_route =
                    utf8_percent_encode(&req.uri().path().as_str(), NON_ALPHANUMERIC).to_string();
                let redirect_url = format!(
                    "/accounts/login?message={}&redirect={}",
                    encoded_msg, encoded_route
                );

                // Delegate to Rocket's Redirect responder
                Redirect::to(redirect_url).respond_to(req)
            }

            OpenFabErr::MissingConfigOption(route, _option) => {
                let (_status, error_msg) = self.match_status_message();

                let encoded_msg = utf8_percent_encode(&error_msg, NON_ALPHANUMERIC).to_string();
                let redirect_url = format!("{}?message={}", route, encoded_msg);

                // Delegate to Rocket's Redirect responder
                Redirect::to(redirect_url).respond_to(req)
            }

            // For other errors, you can map them or fallback to an Internal Server Error response
            OpenFabErr::UuidParseErr(_) => {
                let msg = self.to_string();
                Response::build()
                    .status(Status::BadRequest)
                    .sized_body(msg.len(), std::io::Cursor::new(msg))
                    .ok()
            }
            _ => {
                let (_status, error_msg) = self.match_status_message();
                let encoded_msg = utf8_percent_encode(&error_msg, NON_ALPHANUMERIC).to_string();
                let encoded_route =
                    utf8_percent_encode(&req.uri().path().as_str(), NON_ALPHANUMERIC).to_string();

                // Delegate to Rocket's Redirect responder
                Redirect::to(format!(
                    "/admin/bugreport?error={}&route={}",
                    encoded_msg, encoded_route
                ))
                .respond_to(req)
            }
        }
    }
}

/// Authenticaitn Failure caused by missing user, or invalid password
#[derive(Debug, Error)]
pub enum AuthErr {
    /// password hash didn't verify
    #[error("invalid password entry")]
    InvalidPassword,
    /// user was not found
    #[error("user not found in database")]
    UserNotFound,
}
