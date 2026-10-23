use ormlite::Model;
use ormlite::model::{Join, JoinMeta};
use ormlite::postgres::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
#[ormlite(table = "groups")]
pub struct GroupORM {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub name: String,
    pub description: Option<String>,
}

impl GroupORM {
    pub async fn list_for_user(pool: &PgPool, user: Uuid) -> Result<Vec<GroupORM>, ormlite::Error> {
        let mut conn = pool.acquire().await?;
        Ok(GroupMembershipRel::select()
            .join(GroupMembershipRel::account_group())
            .where_("account = ?")
            .bind(user)
            .fetch_all(&mut *conn)
            .await?
            .into_iter()
            .map(|v: GroupMembershipRel| (*v.account_group).clone())
            .collect::<Vec<GroupORM>>())
    }
}

#[derive(Debug, Model)]
#[ormlite(table = "group_membership")]
pub struct GroupMembershipRel {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    pub account: Uuid,
    #[ormlite(column = "account_group")]
    pub account_group: Join<GroupORM>,
}
