use crate::wrapper::error::ServiceError;
use debil_mysql::DebilConn;

pub struct ConnPool(pub mysql_async::Pool);

impl ConnPool {
    pub fn new(db_url: &str) -> Result<Self, ServiceError> {
        let pool = ConnPool(mysql_async::Pool::from_url(db_url).map_err(|err| {
            ServiceError::bad_request(failure::Error::from_boxed_compat(Box::new(err)))
        })?);

        Ok(pool)
    }

    pub async fn get_conn(&self) -> Result<DebilConn, ServiceError> {
        let conn = self.0.get_conn().await.map_err(|err| {
            ServiceError::internal_server_error(failure::Error::from_boxed_compat(Box::new(err)))
        })?;

        Ok(DebilConn::from_conn(conn))
    }
}
