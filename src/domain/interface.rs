use crate::base64::Base64;
use crate::domain::model::{
    GachaEvent, GachaType, Gift, GiftId, GiftStatus, JankenEvent, JankenStatus,
    PointDiffRankingRecord, PointEvent, PointRankingRecord, User, UserId,
};
use crate::unixtime::UnixTime;
use crate::url::Url;
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;

#[async_trait]
pub trait IUserRepository {
    async fn list_id(&self) -> Result<Vec<UserId>, ServiceError>;
    async fn find_by_id(&self, user_id: &UserId) -> Result<User, ServiceError>;
    async fn find_by_screen_name(&self, screen_name: &String) -> Result<User, ServiceError>;
    async fn find_by_subject(&self, subject: &str) -> Result<User, ServiceError>;
    async fn create(&self, user: User) -> Result<(), ServiceError>;
    async fn save(&self, user: User) -> Result<(), ServiceError>;
    async fn conditional_save_point(
        &self,
        user: User,
        daily_gacha_timestamp: UnixTime,
    ) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IGachaEventRepository {
    async fn find_by_user_type(
        &self,
        user_id: &UserId,
        gacha_type: &GachaType,
    ) -> Result<GachaEvent, ServiceError>;
    async fn create(&self, event: GachaEvent) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IGiftRepository {
    async fn find_by_id(&self, gift_id: &GiftId, user_id: &UserId) -> Result<Gift, ServiceError>;
    async fn find_by_user_id_status(
        &self,
        user_id: &UserId,
        status: GiftStatus,
    ) -> Result<Vec<Gift>, ServiceError>;
    async fn create(&self, gift: Gift) -> Result<(), ServiceError>;
    async fn save_status(
        &self,
        gift_id: GiftId,
        user_id: UserId,
        status: GiftStatus,
    ) -> Result<(), ServiceError>;
    async fn create_for(
        &self,
        gift: Gift,
        users: Vec<UserId>,
        status: GiftStatus,
    ) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IUserIconUploader {
    async fn upload_user_icon(&self, user_id: &UserId, image: Base64) -> Result<Url, ServiceError>;
}

#[async_trait]
pub trait IJankenEventRepository {
    async fn find_by_user_id_status(
        &self,
        user_id: &UserId,
        status: JankenStatus,
    ) -> Result<Vec<JankenEvent>, ServiceError>;
    async fn find_by_user_id(
        &self,
        user_id: &UserId,
        limit: i32,
    ) -> Result<Vec<JankenEvent>, ServiceError>;
    async fn scan_by_status(
        &self,
        status: JankenStatus,
        limit: i32,
    ) -> Result<Vec<JankenEvent>, ServiceError>;
    async fn create(&self, janken_event: JankenEvent) -> Result<(), ServiceError>;
    async fn save(&self, janken_event: JankenEvent) -> Result<(), ServiceError>;
    async fn save_all(&self, janken_events: Vec<JankenEvent>) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IPointEventRepository {
    async fn find_by_id(&self, user_id: &UserId) -> Result<PointEvent, ServiceError>;
    async fn save(&self, event: PointEvent) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IRankingRepository {
    async fn list_top_points(&self, limit: u64) -> Result<Vec<PointRankingRecord>, ServiceError>;
    async fn list_top_point_diffs(
        &self,
        limit: u64,
    ) -> Result<Vec<PointDiffRankingRecord>, ServiceError>;
}
