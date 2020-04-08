use crate::domain::interface::{IGiftRepository, IJankenEventRepository};
use crate::domain::model::{Gift, GiftType, JankenEvent, JankenResult, JankenStatus};
use crate::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use std::sync::Arc;

pub struct JankenProcessService {
    janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
    gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
}

impl JankenProcessService {
    pub fn new(
        janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
        gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
    ) -> Self {
        JankenProcessService {
            janken_repo,
            gift_repo,
        }
    }

    pub async fn process(&self, events: Vec<JankenEvent>) -> Result<(), ServiceError> {
        let mut events_filtered = Vec::new();
        for event in events {
            // タイムアウトを6時間にする
            if (UnixTime::now().datetime_jst() - event.created_at.datetime_jst())
                >= chrono::Duration::hours(6)
            {
                let gift = Gift::new(
                    GiftType::Point(event.point * 2),
                    "じゃんけんで不戦勝となったのでその報酬です".to_string(),
                );
                self.gift_repo.create(gift.clone()).await?;
                self.gift_repo
                    .save_status(gift.id, event.user_id, gift.status)
                    .await?;

                continue;
            } else {
                events_filtered.push(event);
            }
        }

        for chunk in events_filtered.chunks(2) {
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

                    // 勝った方にはギフトとして合計ポイントを送る
                    // 負けた方は、すでにポイントを払っているため何もしない
                    let gift = Gift::new(
                        GiftType::Point(event1.point + event2.point),
                        format!("じゃんけんに勝った報酬です"),
                    );
                    self.gift_repo.create(gift.clone()).await?;
                    self.gift_repo
                        .save_status(gift.id, winner.user_id, gift.status)
                        .await?;
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
    use crate::domain::model::{GiftStatus, JankenEventId, JankenHand, User, UserId};
    use crate::infra::gift_repository_mock::GiftRepositoryMock;
    use crate::infra::janken_event_repository_mock::JankenEventRepositoryMock;
    use crate::infra::user_repository_mock::UserRepositoryStub;
    use crate::unixtime::UnixTime;

    #[tokio::test]
    async fn test_process() -> Result<(), ServiceError> {
        let janken_repo = Arc::new(JankenEventRepositoryMock::new(Vec::new()));
        let gift_repo = Arc::new(GiftRepositoryMock::new());
        let service = JankenProcessService::new(janken_repo.clone(), gift_repo.clone());

        let event_rock = JankenEventId::new();
        let event_paper = JankenEventId::new();
        let event_scissors = JankenEventId::new();

        let user_winner = UserId::new();
        let user_timed_out = UserId::new();

        service
            .process(vec![
                // タイムアウトになるもの
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: user_timed_out.clone(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime(1),
                    status: JankenStatus::Ready,
                    point: 5,
                },
                JankenEvent {
                    id: event_rock.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Rock,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                },
                JankenEvent {
                    id: event_paper.clone(),
                    user_id: user_winner.clone(),
                    hand: JankenHand::Paper,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                },
                JankenEvent {
                    id: event_scissors.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                },
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                },
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
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

        let gifts = gift_repo.created.lock().unwrap().clone();
        assert_eq!(gifts.len(), 2);
        assert_eq!(gifts[0].gift_type, GiftType::Point(10));
        assert_eq!(gifts[1].gift_type, GiftType::Point(10));

        let statuses = gift_repo.saved.lock().unwrap().clone();
        assert_eq!(statuses.len(), 2);
        assert_eq!(statuses[0].1, user_timed_out.clone());
        assert_eq!(statuses[1].1, user_winner.clone());
        assert_eq!(statuses[1].2, GiftStatus::Ready);

        Ok(())
    }
}
