use crate::domain::interface::{IJankenEventRepository, IUserRepository};
use crate::domain::model::{Authorization, JankenEvent, JankenHand, JankenStatus};
use crate::wrapper::error::ServiceError;
use serde::*;
use std::sync::Arc;

pub struct JankenService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
    janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
}

#[derive(Deserialize)]
pub struct JankenCreateInput {
    hand: JankenHand,
}

impl JankenService {
    pub fn new(
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
        janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
    ) -> Self {
        JankenService {
            user_repo,
            janken_repo,
        }
    }

    pub async fn create(
        &self,
        auth: Authorization,
        input: JankenCreateInput,
    ) -> Result<(), ServiceError> {
        let auth_user = auth.require_auth()?;
        let user = self.user_repo.find_by_subject(&auth_user.subject).await?;

        // みょんポイントが5ポイント未満だと出来ない
        if user.point < 5 {
            return Err(ServiceError::bad_request(failure::err_msg(
                "You do not have enough myon point",
            )));
        }

        // 準備中のじゃんけんが残っていたら引けなくする
        let events = self
            .janken_repo
            .find_by_user_id_status(&user.id, JankenStatus::Ready)
            .await?;
        if !events.is_empty() {
            return Err(ServiceError::bad_request(failure::err_msg(
                "Janken Rate Limit Exceeded",
            )));
        }

        let janken = JankenEvent::new(user.id, input.hand);
        self.janken_repo.create(janken).await?;

        Ok(())
    }

    pub async fn find_by_user_id(
        &self,
        auth: Authorization,
    ) -> Result<serde_json::Value, ServiceError> {
        let auth_user = auth.require_auth()?;
        let user = self.user_repo.find_by_subject(&auth_user.subject).await?;

        let events = self.janken_repo.find_by_user_id(&user.id).await?;
        Ok(serde_json::json!({ "events": events }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::janken_event_repository_mock::JankenEventRepositoryMock;
    use crate::infra::user_repository_mock::UserRepositoryStub;

    #[tokio::test]
    async fn create_should_fail_if_previous_is_still_ready() {
        let user_repo = Arc::new(UserRepositoryStub::new(Default::default()));
        let janken_repo = Arc::new(JankenEventRepositoryMock::new(vec![JankenEvent {
            id: Default::default(),
            user_id: Default::default(),
            hand: JankenHand::Rock,
            created_at: Default::default(),
            status: JankenStatus::Ready,
        }]));
        let service = JankenService {
            user_repo: user_repo.clone(),
            janken_repo: janken_repo.clone(),
        };

        let err = service
            .create(
                Authorization::new(Ok(Default::default())),
                JankenCreateInput {
                    hand: JankenHand::Rock,
                },
            )
            .await
            .expect_err("expect error");
        assert_eq!(err.status_code, http::StatusCode::BAD_REQUEST);
    }
}
