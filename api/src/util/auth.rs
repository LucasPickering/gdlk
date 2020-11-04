use crate::error::{
    ActixClientError, ApiError, ApiResult, ClientError, ServerError,
};
use actix_identity::Identity;
use openidconnect::{
    core::{CoreClient, CoreIdTokenClaims},
    AuthorizationCode, CsrfToken, Nonce, TokenResponse,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Data related to a user's current auth state. This is meant to be serialized
/// and stored within the identity cookie that we get from actix-identity. That
/// cookie is signed+encrypted, so any data we put in here is guaranteed to be
/// secret and authentic. This is meant to be used in tandem with
/// [actix-identity::Identity]; this struct provides the data, `Identity`
/// provides the machinery to read from/write to the cookie securely.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IdentityState {
    /// The state that we care about while the user is in the OIDC flow. This
    /// variant is only used while a user is in the login process. Once login
    /// is finished, we should replace the cookie with a `PostAuth` value.
    DuringAuth {
        /// Cross-site Request Forgery token. Used to reject unsolicited OIDC
        /// callbacks
        /// https://auth0.com/docs/protocols/state-parameters
        csrf_token: CsrfToken,
        /// The route to redirect the user to after finishing login
        next: Option<String>,
    },
    /// State that we track when the user is already logged in.
    PostAuth {
        /// The ID of the row in the `user_providers` table that the user is
        /// logged in through. From this we can fetch the `users` row, which
        /// will give us everything we need to know about the user.
        user_provider_id: Uuid,
    },
}

impl IdentityState {
    /// Read the identity state from the identity cookie. If the cookie isn't
    /// present/valid, or if the contents aren't deserializable, return `None`.
    pub fn from_identity(identity: &Identity) -> Option<Self> {
        let id_string = identity.identity()?;
        serde_json::from_str(&id_string).ok()
    }

    /// Serialize this object into a string. Used to store the value in the
    /// identity cookie.
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    /// Check the stored CSRF token against a token that was given as input.
    /// This should be done during the callback stage of auth, to make sure that
    /// the callback is coming from the user that requested the auth cycle.
    pub fn verify_csrf(&self, suspect_csrf_token: &str) -> ApiResult<()> {
        match self {
            // Check that we have a stored token, and that it matches the given
            Self::DuringAuth { csrf_token, .. }
                if suspect_csrf_token == csrf_token.secret() =>
            {
                Ok(())
            }
            _ => Err(ClientError::CsrfError.into()),
        }
    }

    /// Get the `next` param, which tells the API which route to redirect to
    /// after finishing the auth process. If the value isn't present in the
    /// state, just return the root route.
    pub fn next(&self) -> &str {
        match self {
            Self::DuringAuth {
                next: Some(next), ..
            } => next,
            _ => "/",
        }
    }

    /// Get the `user_provider_id` stored in the cookie. This is used to
    /// determine if a user is already logged in.
    pub fn user_provider_id(&self) -> Option<Uuid> {
        match self {
            Self::PostAuth {
                user_provider_id, ..
            } => Some(*user_provider_id),
            _ => None,
        }
    }
}
