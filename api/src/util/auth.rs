use crate::error::{
    ActixClientError, ApiError, ApiResult, ClientError, ServerError,
};
use openidconnect::{
    core::{CoreClient, CoreIdTokenClaims},
    AuthorizationCode, CsrfToken, Nonce, TokenResponse,
};
use serde::{Deserialize, Serialize};

/// HTTP client for OpenID Connect requests. Basically just a wrapper around
/// the Actix HTTP client.
#[allow(unused)]
pub async fn oidc_http_client(
    request: openidconnect::HttpRequest,
) -> Result<openidconnect::HttpResponse, ActixClientError> {
    // There are some unfortunate clones in here because the types we have don't
    // always allow moving the data.

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
    let body: Vec<u8> = actix_response.body().await?.into_iter().collect();

    Ok(openidconnect::HttpResponse {
        status_code,
        headers,
        body,
    })
}

/// Exchanges an authorization code  from the initial login in the for an
/// access token. The code here should come from the browser, which is passed
/// along from the provider. This will make a request to the provider for the
/// exchange.
pub async fn oidc_request_token(
    oidc_client: &CoreClient,
    code: &str,
) -> ApiResult<CoreIdTokenClaims> {
    // Exchange the temp code for a token
    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(code.into()))
        .request_async(oidc_http_client)
        .await
        .map_err(ApiError::from_client_error)?;

    // Verify the response token and get the claims out of it
    let token_verifier = oidc_client.id_token_verifier();
    match token_response.id_token() {
        // I'm not really sure what would cause this, hopefully only a provider
        // bug? We should always get this back because of how we make the
        // request.
        None => Err(ServerError::MissingIdToken.into()),
        Some(id_token) => {
            // TODO better nonce handling here
            match id_token.claims(&token_verifier, &Nonce::new("4".into())) {
                Ok(claims) => Ok(claims.clone()),
                Err(source) => Err(ApiError::from_client_error(source)),
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
            // https://github.com/LucasPickering/gdlk/issues/160
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
    pub fn deserialize(input: Option<&str>) -> ApiResult<Self> {
        match input {
            None => Err(ClientError::CsrfError.into()),
            Some(json_str) => {
                let state: Self = serde_json::from_str(json_str)
                    .map_err(ApiError::from_client_error)?;
                // TODO make this secure
                if "3" == state.csrf_token.secret() {
                    Ok(state)
                } else {
                    Err(ClientError::CsrfError.into())
                }
            }
        }
    }
}
