use base64::{engine::general_purpose, Engine};
use chrono::NaiveDateTime;
use rand::{rngs::OsRng, RngCore};
use rocket::http::private::cookie::Expiration::Session;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use super::SessionClock;
use super::{AuthenticatedPrincipal, SessionConfig, SessionError, SessionId};
use crate::auth::session::clock;
use crate::member::models::MemberId;

/// Represents the claims contained in an access token, including subject, session ID, issuer, audience, issued-at time, and expiration time
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct AccessClaims {
    /// The subject of the token, typically the member ID
    pub(super) sub: MemberId,
    /// The session ID associated with the access token
    pub(super) sid: SessionId,
    /// The issuer of the token, typically the application name or URL
    pub(super) iss: String,
    /// The audience of the token, typically the intended recipients or services
    pub(super) aud: String,
    /// The issued-at timestamp of the token, in seconds since the epoch
    pub(super) iat: i64,
    /// The expiration timestamp of the token, in seconds since the epoch
    pub(super) exp: i64,
}

#[derive(Clone)]
pub(super) struct AccessTokenCodec {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    pub(super) config: SessionConfig,
}

impl AccessTokenCodec {
    pub(super) fn new(secret: &[u8], config: SessionConfig) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            config,
        }
    }

    pub(super) fn issue(
        &self,
        member_id: MemberId,
        session_id: SessionId,
    ) -> Result<(String, NaiveDateTime), SessionError> {
        let (now, expires_at) = SessionClock::expires_in(self.config.access_ttl);
        let claims = AccessClaims {
            sub: member_id,
            sid: session_id,
            iss: self.config.issuer.clone(),
            aud: self.config.audience.clone(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
        };

        let token = encode(&Header::new(Algorithm::HS512), &claims, &self.encoding_key)?;
        Ok((token, expires_at.naive_utc()))
    }

    pub(super) fn verify(&self, token: &str) -> Result<AuthenticatedPrincipal, SessionError> {
        let mut validation = Validation::new(Algorithm::HS512);
        validation.leeway = 0;
        validation.set_issuer(&[self.config.issuer.as_str()]);
        validation.set_audience(&[self.config.audience.as_str()]);
        validation.required_spec_claims = HashSet::from([
            "sub".to_string(),
            "aud".to_string(),
            "iss".to_string(),
            "exp".to_string(),
        ]);

        let claims = decode::<AccessClaims>(token, &self.decoding_key, &validation)?.claims;

        let now = SessionClock::now().timestamp();
        let clock_skew = 30i64;
        if claims.iat > now + clock_skew {
            log::error!(
                "Access token issued in the future: iat={}, now={}",
                claims.iat,
                now
            );
            return Err(SessionError::ExpiredAccessToken);
        }

        Ok(AuthenticatedPrincipal {
            member_id: claims.sub,
            session_id: claims.sid,
        })
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct RefreshTokenCodec;
impl RefreshTokenCodec {
    /// Issues a new refresh token and its hash.
    pub(super) fn issue() -> (String, String) {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);

        let token = general_purpose::URL_SAFE_NO_PAD.encode(&bytes);
        let hash = Self::hash(&token);
        (token, hash)
    }

    pub(super) fn hash(token: &str) -> String {
        blake3::hash(token.as_bytes()).to_string()
    }
}
