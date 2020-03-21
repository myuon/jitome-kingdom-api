use crate::domain::model::AuthUser;
use crate::wrapper::error::ServiceError;
use biscuit::errors::Error;
use serde::*;
use std::sync::Arc;

impl From<biscuit::errors::Error> for ServiceError {
    fn from(err: Error) -> Self {
        ServiceError::unauthorized(failure::Error::from_boxed_compat(Box::new(err)))
    }
}

// for parsing payload in jwt
#[derive(Serialize, Deserialize)]
struct CustomPayload {}

pub struct JWTHandler {
    public_key: Arc<biscuit::jwk::JWKSet<biscuit::Empty>>,
}

impl JWTHandler {
    pub fn new(public_key: Arc<biscuit::jwk::JWKSet<biscuit::Empty>>) -> Self {
        JWTHandler { public_key }
    }

    pub async fn load_from_jwk(jwk_url: &str) -> biscuit::jwk::JWKSet<biscuit::Empty> {
        reqwest::get(jwk_url).await.unwrap().json().await.unwrap()
    }

    fn get_key_from_jwk(
        &self,
        kid: &str,
    ) -> (biscuit::jws::Secret, biscuit::jwa::SignatureAlgorithm) {
        let key = self.public_key.find(kid).unwrap().clone();

        match key.algorithm {
            biscuit::jwk::AlgorithmParameters::RSA(params) => (
                params.jws_public_key_secret(),
                biscuit::jwa::SignatureAlgorithm::RS256,
            ),
            _ => unimplemented!(),
        }
    }

    pub fn authorize(&self, auth_token: &str) -> Result<AuthUser, ServiceError> {
        let token = auth_token.split("Bearer ").collect::<Vec<&str>>();
        if token.len() != 2 {
            return Err(ServiceError::unauthorized(failure::err_msg(
                "access denied",
            )));
        }

        self.verify(token[1])
    }

    fn verify(&self, jwt: &str) -> Result<AuthUser, ServiceError> {
        let jwt = biscuit::JWT::<CustomPayload, biscuit::Empty>::new_encoded(jwt);
        let kid = jwt
            .unverified_header()?
            .registered
            .key_id
            .ok_or(failure::err_msg("None for key_id"))?;

        let jwt = jwt.decode_with_jwks(self.public_key.as_ref())?;
        jwt.validate(Default::default())?;

        let payload = jwt.payload()?.clone();
        Ok(AuthUser {
            subject: payload
                .registered
                .subject
                .as_ref()
                .ok_or(ServiceError::bad_request(failure::err_msg("no subject")))?
                .to_string(),
        })
    }
}
