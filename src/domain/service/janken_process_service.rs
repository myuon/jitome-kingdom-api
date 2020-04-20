use crate::domain::interface::{IGiftRepository, IJankenEventRepository, IUserRepository};
use crate::domain::model::{Gift, GiftType, JankenEvent, JankenResult, JankenStatus};
use crate::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use rand::seq::SliceRandom;
use std::sync::Arc;

pub struct JankenProcessService {
    janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
    gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
}

impl JankenProcessService {
    pub fn new(
        janken_repo: Arc<dyn IJankenEventRepository + Sync + Send>,
        gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
    ) -> Self {
        JankenProcessService {
            janken_repo,
            gift_repo,
            user_repo,
        }
    }

    pub async fn process(&self, events: Vec<JankenEvent>) -> Result<(), ServiceError> {
        let mut events_filtered = Vec::new();
        for mut event in events {
            // タイムアウトを設定
            if (UnixTime::now().datetime_jst() - event.created_at.datetime_jst())
                >= chrono::Duration::hours(8)
            {
                event.set_timeout();
                self.janken_repo.save(event.clone()).await?;

                let gift = Gift::new(
                    GiftType::Point(event.point * 2),
                    "じゃんけんで不戦勝となったのでその報酬です".to_string(),
                );
                let status = gift.status.clone();
                self.gift_repo
                    .create_for(gift, vec![event.user_id], status)
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

                    let winner_user = self.user_repo.find_by_id(&winner.user_id).await?;
                    let loser_user = self.user_repo.find_by_id(&loser.user_id).await?;

                    winner.set_opponent(loser_user.id, loser_user.screen_name);
                    loser.set_opponent(winner_user.id, winner_user.screen_name);

                    self.janken_repo
                        .save_all(vec![winner.clone(), loser.clone()])
                        .await?;

                    // 勝った方にはギフトとして合計ポイントを送る
                    // 負けた方は、すでにポイントを払っているため何もしない
                    let mut gift = Gift::new(
                        GiftType::Point(event1.point + event2.point),
                        format!("じゃんけんに勝った報酬です"),
                    );

                    // じゃんけんのイベントIDを追跡用に紐付けておくことで、途中で落ちたときに追跡できるようにしておく
                    gift.set_janken_events(winner.id, loser.id);

                    let status = gift.status.clone();
                    self.gift_repo
                        .create_for(gift, vec![winner.user_id], status)
                        .await?;
                }
                _ => break,
            }
        }

        Ok(())
    }

    pub async fn run(&self) -> Result<(), ServiceError> {
        loop {
            let mut events = self
                .janken_repo
                .scan_by_status(JankenStatus::Ready, 100)
                .await?;

            // あいこはスルーされる仕組みなので、適当にランダマイズしないと延々待たされる待たされる可能性がある
            events.shuffle(&mut rand::thread_rng());

            self.process(events).await?;

            // 30秒くらい待つ
            tokio::time::delay_for(tokio::time::Duration::from_secs(30)).await;
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
        let user_repo = Arc::new(UserRepositoryStub::new(Default::default()));
        let service =
            JankenProcessService::new(janken_repo.clone(), gift_repo.clone(), user_repo.clone());

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
                    opponent_user_id: None,
                    opponent_user_screen_name: None,
                },
                JankenEvent {
                    id: event_rock.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Rock,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                    opponent_user_id: None,
                    opponent_user_screen_name: None,
                },
                JankenEvent {
                    id: event_paper.clone(),
                    user_id: user_winner.clone(),
                    hand: JankenHand::Paper,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                    opponent_user_id: None,
                    opponent_user_screen_name: None,
                },
                JankenEvent {
                    id: event_scissors.clone(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                    opponent_user_id: None,
                    opponent_user_screen_name: None,
                },
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                    opponent_user_id: None,
                    opponent_user_screen_name: None,
                },
                JankenEvent {
                    id: JankenEventId::new(),
                    user_id: UserId::new(),
                    hand: JankenHand::Scissors,
                    created_at: UnixTime::now(),
                    status: JankenStatus::Ready,
                    point: 5,
                    opponent_user_id: None,
                    opponent_user_screen_name: None,
                },
            ])
            .await?;

        let events = janken_repo.saved.lock().unwrap().clone();
        // あいこの場合は何も起きないので決着が着くのは2つ, タイムアウトになるものが1つで合計3つ
        assert_eq!(events.len(), 3);

        // Paperのイベントが勝つ
        assert_eq!(events[1].id, event_paper);
        assert_eq!(events[1].status, JankenStatus::Won);

        // Rockのイベントは負ける
        assert_eq!(events[2].id, event_rock);
        assert_eq!(events[2].status, JankenStatus::Lost);

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
