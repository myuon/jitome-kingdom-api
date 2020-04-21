use crate::domain::interface::{IGachaEventRepository, IUserRepository};
use crate::domain::model::{Authorization, GachaEvent, GachaEventId, GachaType};
use crate::error::ServiceError;
use crate::wrapper::rand_gen::RandomGen;
use crate::wrapper::unixtime::UnixTime;
use serde::*;
use std::sync::Arc;

// ガチャ
pub struct GachaService {
    gacha_repo: Arc<dyn IGachaEventRepository + Sync + Send>,
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
}

#[derive(Serialize)]
pub struct DailyGachaRecord {
    latest: Option<GachaEvent>,
    is_available: bool,
    next_gacha_time: UnixTime,
}

impl GachaService {
    pub fn new(
        gacha_repo: Arc<dyn IGachaEventRepository + Sync + Send>,
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
    ) -> GachaService {
        GachaService {
            gacha_repo,
            user_repo,
        }
    }

    pub async fn get_latest_daily_event(
        &self,
        auth: Authorization,
    ) -> Result<serde_json::Value, ServiceError> {
        let auth_user = auth.require_auth()?;
        let user = self.user_repo.find_by_subject(&auth_user.subject).await?;

        match self
            .gacha_repo
            .find_by_user_type(&user.id, &GachaType::Daily)
            .await
        {
            Ok(r) => serde_json::to_value(&r).map_err(|err| {
                ServiceError::internal_server_error(failure::Error::from_boxed_compat(Box::new(
                    err,
                )))
            }),
            Err(err) if err.status_code == http::StatusCode::NOT_FOUND => {
                Ok(serde_json::json!(null))
            }
            Err(err) => Err(err),
        }
    }

    pub async fn get_daily_gacha_record(
        &self,
        auth: Authorization,
    ) -> Result<DailyGachaRecord, ServiceError> {
        let auth_user = auth.require_auth()?;
        let user = self.user_repo.find_by_subject(&auth_user.subject).await?;

        let latest = match self
            .gacha_repo
            .find_by_user_type(&user.id, &GachaType::Daily)
            .await
        {
            Err(err) if err.status_code == http::StatusCode::NOT_FOUND => Ok(None),
            r => r.map(|e| Some(e)),
        }?;

        Ok(DailyGachaRecord {
            latest,
            is_available: user.is_daily_gacha_available_at(UnixTime::now()),
            next_gacha_time: UnixTime::now(),
        })
    }

    pub async fn try_daily(&self, auth: Authorization) -> Result<serde_json::Value, ServiceError> {
        let auth_user = auth.require_auth()?;
        let mut user = self.user_repo.find_by_subject(&auth_user.subject).await?;
        let user_cloned = user.clone();

        if !user.is_daily_gacha_available_at(UnixTime::now()) {
            return Err(ServiceError::bad_request(failure::err_msg(
                "Daily Gacha Rate Limit Exceeded",
            )));
        }

        // 終端の16は含まない
        let n = RandomGen::range(5, 16);
        user.add_point(n);
        let prev_timestamp = user.update_daily_gacha_timestamp();

        // ここでデイリーガチャのタイムスタンプでconditional writeを行うことで競合を防ぐ
        self.user_repo
            .conditional_save_point(user.clone(), prev_timestamp)
            .await?;

        let event = GachaEvent {
            id: GachaEventId::new(),
            user_id: user.id,
            gacha_type: GachaType::Daily,
            created_at: UnixTime::now(),
        };

        if let Err(err) = self.gacha_repo.create(event.clone()).await {
            warn!("Failed to create a new gacha event: {:?} {:?}", event, err);
            error!("{:?}", err);

            // 失敗したときはロールバックを試みる
            if let Err(err) = self.user_repo.save(user_cloned.clone()).await {
                // ロールバックに失敗した場合は不整合が起こるのでログだけ吐いておく
                error!("Failed to save the original user data: {:?}", user_cloned);
                error!("{:?}", err);

                return Err(ServiceError::internal_server_error(failure::err_msg(
                    "operation failed",
                )));
            }

            warn!("Rollback completed");

            return Err(err);
        };

        Ok(serde_json::json!({ "obtained": n }))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::domain::model::{User, UserId};
    use crate::infra::gacha_event_repository_mock::*;
    use crate::infra::user_repository_mock::*;

    #[tokio::test]
    async fn gacha_available_with_no_records() -> Result<(), ServiceError> {
        let user_id = UserId::new();
        let service = GachaService::new(
            Arc::new(GachaEventRepositoryStub::new_empty()),
            Arc::new(UserRepositoryStub::new(User {
                id: user_id.clone(),
                last_tried_daily_gacha: UnixTime(0),
                ..Default::default()
            })),
        );

        let record = service
            .get_daily_gacha_record(Authorization::new(Ok(Default::default())))
            .await?;
        assert!(record.is_available);
        assert_eq!(record.latest, None);

        Ok(())
    }

    #[tokio::test]
    async fn gacha_available_with_old_record() -> Result<(), ServiceError> {
        let user_id = UserId::new();
        let event = GachaEvent {
            gacha_type: GachaType::Daily,
            created_at: UnixTime(0),
            ..Default::default()
        };
        let service = GachaService::new(
            Arc::new(GachaEventRepositoryStub::new(event.clone())),
            Arc::new(UserRepositoryStub::new(User {
                id: user_id.clone(),
                last_tried_daily_gacha: UnixTime(0),
                ..Default::default()
            })),
        );

        let record = service
            .get_daily_gacha_record(Authorization::new(Ok(Default::default())))
            .await?;
        assert!(record.is_available);
        assert_eq!(record.latest, Some(event.clone()));

        Ok(())
    }

    #[tokio::test]
    async fn gacha_not_available_with_todays_record() -> Result<(), ServiceError> {
        let user_id = UserId::new();
        let service = GachaService::new(
            Arc::new(GachaEventRepositoryStub::new(GachaEvent {
                gacha_type: GachaType::Daily,
                created_at: UnixTime::now(),
                ..Default::default()
            })),
            Arc::new(UserRepositoryStub::new(User {
                id: user_id.clone(),
                last_tried_daily_gacha: UnixTime::now(),
                ..Default::default()
            })),
        );

        let record = service
            .get_daily_gacha_record(Authorization::new(Ok(Default::default())))
            .await?;
        assert!(!record.is_available);

        Ok(())
    }

    #[tokio::test]
    async fn cannot_try_gacha_if_already_tried_in_same_day() -> Result<(), ServiceError> {
        let service = GachaService::new(
            Arc::new(GachaEventRepositoryStub::new(GachaEvent::default())),
            Arc::new(UserRepositoryStub::new(User {
                last_tried_daily_gacha: UnixTime::now(),
                ..Default::default()
            })),
        );

        let err = service
            .try_daily(Authorization::new(Ok(Default::default())))
            .await
            .expect_err("expect error");
        assert_eq!(err.status_code, http::StatusCode::BAD_REQUEST);

        Ok(())
    }
}
