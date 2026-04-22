use openfab::oidc::OidcClaims;
use rocket_oidc::auth::ApiKeyGuard;
use rocket_dyn_templates::tera::Context;

pub type Guard = ApiKeyGuard<OidcClaims>;

pub fn global_context() -> Context {
    Context::new()
}