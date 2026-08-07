//! v1 principal contract: `X-Principal` header (unverified — SPEC-014).

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

#[derive(Debug, Clone, Default)]
pub struct Principal(pub String);

impl<S> FromRequestParts<S> for Principal
where
    S: Send + Sync,
{
    type Rejection = crate::error::ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-principal")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "anonymous".to_string());
        Ok(Principal(value))
    }
}
