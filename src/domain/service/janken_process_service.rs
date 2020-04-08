use crate::domain::interface::{IJankenEventRepository, IUserRepository};
use crate::domain::model::{JankenEvent, JankenResult, JankenStatus};
use crate::error::ServiceError;
use std::sync::Arc;

pub struct JankenProcessService {
    janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
}

impl JankenProcessService {
    pub fn new(
        janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
    ) -> Self {
        JankenProcessService {
            janken_repo,
            user_repo,
        }
    }

    pub async fn process(&self, events: Vec<JankenEvent>) -> Result<(), ServiceError> {
        for chunk in events.chunks(2) {
            match chunk {
                [event1, event2] => {
                    let (mut winner, mut loser) = match event1.hand.fight(&event2.hand) {
                        JankenResult::Tie => continue,
                        JankenResult::Win => (event1.clone(), event2.clone()),
                        JankenResult::Lose => (event2.clone(), event1.clone()),
                    };

                    winner.status = JankenStatus::Won;
                    loser.status = JankenStatus::Lost;

                    // トランザクション張ろうね
                    self.janken_repo.save(winner.clone()).await?;
                    self.janken_repo.save(loser.clone()).await?;

                    // 勝ったほうは+5, 負けたほうは-5する
                    let mut winner_user = self.user_repo.find_by_id(&winner.user_id).await?;
                    winner_user.point += 5;
                    self.user_repo.save(winner_user).await?;

                    let mut loser_user = self.user_repo.find_by_id(&loser.user_id).await?;
                    loser_user.point -= 5;
                    self.user_repo.save(loser_user).await?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    pub async fn run(&self) -> Result<(), ServiceError> {
        loop {
            let events = self
                .janken_repo
                .scan_by_status(JankenStatus::Ready, 100)
                .await?;
            self.process(events).await?;

            // 5分くらい待つ
            tokio::time::delay_for(tokio::time::Duration::from_secs(300)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{JankenEventId, JankenHand, User, UserId};
    use crate::infra::janken_event_repository_mock::JankenEventRepositoryMock;
    use crate::infra::user_repository_mock::UserRepositoryStub;
    use crate::unixtime::UnixTime;

    #[tokio::test]
    async fn test_process() -> Result<(), ServiceError> {
        let janken_repo = Arc::new(JankenEventRepositoryMock::new(Vec::new()));
        let user_repo = Arc::new(UserRepositoryStub::new(User {
            point: 100,
            ..Default::default()
        }));
        let service = JankenProcessService::new(janken_repo.clone(), user_repo.clone());

        let event_rock = JankenEventId::new();
        let event_paper = JankenEventId::new();
        let event_scissors = JankenEventId::new();
        service
            .process(vec![
                JankenEvent {
                    id: event_rock.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Rock,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                },
                JankenEvent {
                    id: event_paper.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Paper,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                },
                JankenEvent {
                    id: event_scissors.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                },
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                },
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                },
            ])
            .await?;

        let events = janken_repo.saved.lock().unwrap().clone();
        // あいこの場合は何も起きないので決着が着くのは2つ
        assert_eq!(events.len(), 2);

        // Paperのイベントが勝つ
        assert_eq!(events[0].id, event_paper);
        assert_eq!(events[0].status, JankenStatus::Won);

        // Rockのイベントは負ける
        assert_eq!(events[1].id, event_rock);
        assert_eq!(events[1].status, JankenStatus::Lost);

        let users = user_repo.saved.lock().unwrap().clone();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].point, 105);
        assert_eq!(users[1].point, 95);

        Ok(())
    }
}
