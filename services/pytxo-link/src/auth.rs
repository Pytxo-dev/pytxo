use axum::http::HeaderMap;

use crate::state::AppState;

/// Subject returned after successful auth (Clerk user id or api-key sentinel).
pub struct AuthSubject {
    pub user_id: String,
    pub org_id: Option<String>,
}

/// Validates API key, admin key, or Clerk session bearer.
pub async fn authorized(headers: &HeaderMap, state: &AppState) -> Option<AuthSubject> {
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if state.require_auth && state.api_key.is_none() && state.jwks.is_none() {
        return None;
    }

    if let Some(expected) = state.api_key.as_ref() {
        if bearer == Some(expected.as_str()) {
            return Some(AuthSubject {
                user_id: "api-key".to_string(),
                org_id: None,
            });
        }
    }

    if let (Some(token), Some(jwks)) = (bearer, state.jwks.as_ref()) {
        if let Some(claims) = jwks.validate(token).await {
            let org_id = claims.org_id;
            return Some(AuthSubject {
                user_id: claims.sub,
                org_id,
            });
        }
    }

    if !state.require_auth {
        return Some(AuthSubject {
            user_id: bearer.unwrap_or("anonymous").to_string(),
            org_id: None,
        });
    }

    None
}

pub fn admin_authorized(headers: &HeaderMap, state: &AppState) -> bool {
    let Some(expected) = state.admin_key.as_ref() else {
        return false;
    };
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::trim);
    let header = headers
        .get("x-link-admin-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim);
    bearer == Some(expected.as_str()) || header == Some(expected.as_str())
}
