use openfab::oidc::OidcClaims;
use rocket_oidc::auth::ApiKeyGuard;

pub type Guard = ApiKeyGuard<OidcClaims>;
