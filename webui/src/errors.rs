use crate::discovery::AdvertiseCmd;
use openfab::discovery::IppDiscoveryError;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rocket::{
    Request, Response,
    http::{ContentType, Status},
    response::Redirect,
    response::Responder,
    response::content::RawHtml,
};
use rocket_dyn_templates::{Template, context};
use std::io::Cursor;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum OpenFabErr {
    #[error("failed to send advertisement command: {0}")]
    AdvertiseErr(#[from] SendError<AdvertiseCmd>),

    #[error("failed to discover printers: {0}")]
    IppDiscovery(#[from] IppDiscoveryError),

    #[error("database error: {0}")]
    OrmLite(#[from] ormlite::Error),

    #[error("sql error: {0}")]
    SQLXErr(#[from] ormlite::SqlxError),

    #[error("Minio error: {0}")]
    MinioErr(#[from] minio_rsc::error::Error),

    #[error("Minio value error: {0}")]
    ValueErr(#[from] minio_rsc::error::ValueError),

    #[error("secret fetch error: {0}")]
    SecretFetchErr(#[from] secret_ref::SecretError),

    #[error("authentication failed: {0}")]
    AuthErr(#[from] AuthErr),

    #[error("driver error: {0}")]
    DriverError(#[from] openfab_drivers::errors::DriverError),

    #[error("driver config error: {0}")]
    DriverConfigError(#[from] openfab_drivers::errors::ConfigError),

    #[error("failed to parse string to uuid: {0}")]
    UuidParseErr(#[from] uuid::Error),

    #[error("failed to parse config missing: {1} from {0}")]
    MissingConfigOption(String, String),

    #[error("json serialization error: {0}")]
    JsonErr(#[from] serde_json::Error),
}

impl OpenFabErr {
    pub fn match_status_message(&self) -> (Status, String) {
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
        let route = req.route().map(|v| v.uri.as_str().to_string());
        match &self {
            // If it's an authentication error, redirect back to the login page with the error message
            OpenFabErr::AuthErr(auth_err) => {
                let (status, error_msg) = self.match_status_message();
                let encoded_msg = utf8_percent_encode(&error_msg, NON_ALPHANUMERIC).to_string();

                let redirect_url = format!("/accounts/login?message={}", encoded_msg);

                // Delegate to Rocket's Redirect responder
                Redirect::to(redirect_url).respond_to(req)
            }

            OpenFabErr::MissingConfigOption(route, option) => {
                let (status, error_msg) = self.match_status_message();

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
                let (status, error_msg) = self.match_status_message();
                let encoded_msg = utf8_percent_encode(&error_msg, NON_ALPHANUMERIC).to_string();

                let redirect_url = match route {
                    Some(route) => {
                        format!("/admin/bugreport?error={}&route={}", encoded_msg, route)
                    }
                    None => format!("/admin/bugreport?error={}", encoded_msg),
                };

                // Delegate to Rocket's Redirect responder
                Redirect::to(redirect_url).respond_to(req)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum AuthErr {
    #[error("invalid password entry")]
    InvalidPassword,
    #[error("user not found in database")]
    UserNotFound,
}
