use crate::accounts::User;
use crate::admin::groups::{GroupMembershipRel, GroupORM};
use crate::errors::OpenFabErr;

use ormlite::Model;
use ormlite::model::{Join, JoinMeta};
use ormlite::postgres::PgPool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CREATE_MATERIAL_PROFILE: &'static str = "settings.materials.create";
pub const READ_MATERIAL_PROFILE: &'static str = "settings.materials.read";
pub const UPDATE_MATERIAL_PROFILE: &'static str = "settings.materials.update";
pub const DELETE_MATERIAL_PROFILE: &'static str = "settings.materials.delete";

/*
pub struct PermissionSet {
    pub create: String,
    pub read: String,
    pub update: String,
    pub delete: String,
}

pub struct Resource {
    scope: String,
    name: String,
    permission_set: PermissionSet,
}

impl Resource {
    pub fn new(scope: &str, name: &str) -> Self {
        let set = {
            create: format!("{}.{}.create", scope, name),
            read: format!("{}.{}.read", )
        }
    }
    pub fn permissions(&self) -> PermissionSet {
        self.permission_set.clone()
    }
}*/

#[derive(Debug, Model)]
#[ormlite(table = "user_permissions_rel")]
pub struct GroupPermissionRel {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// e.g. settings.materials.create, settings.materials.read, admin.users.create, admin.groups.create, library.assets.create
    pub name: String,
    /// group that the permission applies to.
    #[ormlite(column = "group")]
    pub group: Join<GroupORM>,
}

#[derive(Debug, Model)]
#[ormlite(table = "user_permissions_rel")]
pub struct UserPermissionRel {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// e.g. settings.materials.create, settings.materials.read, admin.users.create, admin.groups.create, library.assets.create
    pub permission: String,
    #[ormlite(column = "user")]
    pub user: Join<User>,
}

pub async fn check_user_permission(
    pool: &PgPool,
    user: Uuid,
    permission: &str,
) -> Result<bool, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    /// check if the user directly has the requested permission
    let user_perms = UserPermissionRel::select()
        .join(UserPermissionRel::user())
        .where_("user.uid = ? and permission = ?")
        .bind(user)
        .bind(permission)
        .fetch_all(&mut *conn)
        .await?;

    if user_perms.len() > 0 {
        return Ok(true);
    }

    let groups = GroupMembershipRel::select()
        .join(GroupMembershipRel::user())
        .join(GroupMembershipRel::group())
        .where_("user.uid = ?")
        .bind(user)
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|g| g.group.uid)
        .collect::<Vec<Uuid>>();

    let group_perms = GroupPermissionRel::select()
        .join(GroupPermissionRel::group())
        .where_("group.uid in ? and permission = ?")
        .bind(groups)
        .bind(permission)
        .fetch_all(&mut *conn)
        .await?;

    if group_perms.len() > 0 {
        return Ok(true);
    }
    return Ok(false);
}
