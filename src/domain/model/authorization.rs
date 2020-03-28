use crate::wrapper::error::ServiceError;
use serde::*;

#[derive(Clone, PartialOrd, PartialEq)]
pub enum Role {
    Unknown,
    Admin,
}

impl Role {
    pub fn from_str(rep: &str) -> Self {
        match rep {
            "admin" => Role::Admin,
            _ => Role::Unknown,
        }
    }

    pub fn to_string(&self) -> String {
        use Role::*;

        match self {
            Unknown => "unknown",
            Admin => "admin",
        }
        .to_string()
    }
}

impl Serialize for Role {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Default)]
pub struct AuthUser {
    pub subject: String,
    pub roles: Vec<Role>,
}

impl AuthUser {
    pub fn require_admin(&self) -> Result<(), ServiceError> {
        if !self.roles.contains(&Role::Admin) {
            return Err(ServiceError::unauthorized(failure::err_msg(
                "access_denied",
            )));
        }

        Ok(())
    }
}

pub struct Authorization {
    result: Result<AuthUser, ServiceError>,
}

impl Authorization {
    pub fn new(result: Result<AuthUser, ServiceError>) -> Authorization {
        Authorization { result }
    }

    pub fn require_auth(self) -> Result<AuthUser, ServiceError> {
        self.result
    }
}
