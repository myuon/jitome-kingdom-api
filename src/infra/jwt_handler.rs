use crate::domain::interface::IJWTHandler;
use crate::wrapper::error::ServiceError;
use biscuit::errors::Error;
use futures::FutureExt;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;

impl From<biscuit::errors::Error> for ServiceError {
    fn from(err: Error) -> Self {
        ServiceError::internal_server_error(failure::Error::from_boxed_compat(Box::new(err)))
    }
}

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
}

impl<Payload: Serialize + DeserializeOwned + Clone> IJWTHandler<Payload> for JWTHandler {
    fn verify(&self, jwt: &str) -> Result<Payload, ServiceError> {
        let jwt = biscuit::JWT::<Payload, biscuit::Empty>::new_encoded(jwt);
        let kid = jwt
            .unverified_header()?
            .registered
            .key_id
            .ok_or(failure::err_msg("None for key_id"))?;

        let (secret, alg) = self.get_key_from_jwk(&kid);
        let jwt = jwt.into_decoded(&secret, alg)?;
        jwt.validate(Default::default())?;

        Ok(jwt.payload()?.private.clone())
    }
}
