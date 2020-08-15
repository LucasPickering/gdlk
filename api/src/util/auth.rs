use crate::error::{ActixClientError, ResponseError, ResponseResult};
use openidconnect::{
    core::{CoreClient, CoreIdTokenClaims},
    AuthorizationCode, CsrfToken, Nonce,
};
use serde::{Deserialize, Serialize};

/// HTTP client for OpenID Connect requests. Basically just a wrapper around
/// the Actix HTTP client.
pub async fn oidc_http_client(
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::HttpResponse, ActixClientError> {
    // There are some unfortunate clones in here because the types we have don't
    // always allow moving the data.
    // TODO there's probably still room for improvement here

    let client = actix_web::client::Client::default();

    // Convert the OIDC request to an actix request
    let mut actix_request =
        client.request(request.method, request.url.to_string());
    let headers = actix_request.headers_mut();
    for (key, val) in &request.headers {
        headers.insert(key.clone(), val.clone());
    }

    // Get the response and convert it from actix format to OIDC
    let mut actix_response = actix_request.send_body(request.body).await?;
    let status_code = actix_response.status();
    let mut headers = openidconnect::http::HeaderMap::new();
    for (key, val) in actix_response.headers() {
        headers.insert(key, val.clone());
    }
    let body = actix_response.body().await?.into_iter().collect();

    Ok(openidconnect::HttpResponse {
        status_code,
        headers,
        body,
    })
}

/// Exchanges the access token from the initial login in the openid provider
/// for a normal token. The code here should come from the browser, which
/// is passed along from the provider.
pub async fn oidc_request_token(
    oidc_client: &CoreClient,
    code: &str,
) -> Result<CoreIdTokenClaims, ResponseError> {
    // Exchange the temp code for a token
    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(code.into()))
        .request_async(oidc_http_client)
        .await?;

    // Verify the response token and get the claims out of it
    let token_verifier = oidc_client.id_token_verifier();
    match token_response.extra_fields().id_token() {
        None => Err(ResponseError::InvalidCredentials),
        Some(id_token) => {
            // TODO better nonce handling here
            match id_token.claims(&token_verifier, &Nonce::new("4".into())) {
                // TODO remove clone
                Ok(claims) => Ok(claims.clone()),
                Err(_) => Err(ResponseError::InvalidCredentials),
            }
        }
    }
}

/// Contents of the `state` query param that gets passed through the OpenID
/// login. These will be serialized via JSON -> base64.
/// https://auth0.com/docs/protocols/oauth2/oauth-state
#[derive(Serialize, Deserialize, Debug)]
pub struct AuthState {
    /// The next param determines what page to redirect the user to after login
    next: Option<String>,
    csrf_token: CsrfToken,
}

impl AuthState {
    /// Create a new state with the given redirect param.
    pub fn new(next: Option<String>) -> Self {
        Self {
            next,
            // TODO make this secure
            csrf_token: CsrfToken::new("3".into()),
        }
    }

    /// Get the `next` field, which specifies which route to redirect the user
    /// to after the logic is successful.
    pub fn next(&self) -> &str {
        self.next.as_deref().unwrap_or("/")
    }

    /// Serialization into a string that can be passed to `openidconnect`.
    pub fn serialize(&self) -> CsrfToken {
        let json_string = serde_json::to_string(self).unwrap();
        CsrfToken::new(json_string)
    }

    /// Deserialize input from the user into auth state. This will also
    /// validate the CSRF token in the param.
    pub fn deserialize(input: Option<&str>) -> ResponseResult<Self> {
        match input {
            None => Err(ResponseError::InvalidCredentials),
            Some(json_str) => {
                let state: Self = serde_json::from_str(json_str)?;
                // TODO make this secure
                if "3" == state.csrf_token.secret() {
                    Ok(state)
                } else {
                    Err(ResponseError::InvalidCredentials)
                }
            }
        }
    }
}
