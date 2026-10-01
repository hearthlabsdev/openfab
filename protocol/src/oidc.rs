//! OIDC related types and utilities.

/// OIDC claims extracted from the ID token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClaims {
    /// Subject identifier
    pub sub: String,
    /// Issuer identifier
    pub iss: String,
    /// Audience identifier
    pub aud: String,
    /// Expiration time (in seconds since epoch)
    pub exp: i64,
    /// Issued at time (in seconds since epoch)
    pub iat: i64,
    /// Name of the user
    pub name: Option<String>,
    /// Email address of the user
    pub email: Option<String>,
}