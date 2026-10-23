#![allow(unused)]
use crate::accounts::User;
use crate::admin::groups::{GroupMembershipRel, GroupORM};
use crate::errors::AuthErr;
use crate::errors::OpenFabErr;
use ormlite::Model;
use ormlite::model::{Join, JoinMeta};
use ormlite::postgres::PgPool;
use uuid::Uuid;

pub const READ_LIBRARY: &'static str = "library.read";
pub const CREATE_MATERIAL_PROFILE: &'static str = "settings.materials.create";
pub const READ_MATERIAL_PROFILE: &'static str = "settings.materials.read";
pub const UPDATE_MATERIAL_PROFILE: &'static str = "settings.materials.update";
pub const DELETE_MATERIAL_PROFILE: &'static str = "settings.materials.delete";

pub const READ_USERS: &'static str = "admin.users.read";

pub const CREATE_ACTIVATION_CODES: &'static str = "admin.activation-codes.create";
pub const READ_ACTIVATION_CODES: &'static str = "admin.activation-codes.read";
pub const UPDATE_ACTIVATION_CODES: &'static str = "admin.activation-codes.update";
pub const DELETE_ACTIVATION_CODES: &'static str = "admin.activation-codes.delete";
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
#[ormlite(table = "group_permissions_rel")]
pub struct GroupPermissionRel {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// e.g. settings.materials.create, settings.materials.read, admin.users.create, admin.groups.create, library.assets.create
    pub permission: String,
    pub entity: Option<String>,
    /// group that the permission applies to.
    pub account_group: Uuid,
}

#[derive(Debug, Model)]
#[ormlite(table = "user_permissions_rel")]
pub struct UserPermissionRel {
    #[ormlite(primary_key)]
    pub uid: Uuid,
    /// e.g. settings.materials.create, settings.materials.read, admin.users.create, admin.groups.create, library.assets.create
    pub permission: String,
    /// which entity this applies to (e.g. asset.uid, library.uid, etc)
    pub entity: Option<Uuid>,
    //#[ormlite(column = "users")]
    pub account: Uuid,
}

pub async fn check_subject_permission(
    pool: &PgPool,
    subject: &str,
    permission: &str,
    entity: Option<Uuid>,
) -> Result<bool, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    let user = User::select()
        .where_("subject = ?")
        .bind(subject)
        .fetch_one(&mut *conn)
        .await?;
    check_user_permission(pool, user.uid, permission, entity).await
}
pub async fn check_user_permission(
    pool: &PgPool,
    user: Uuid,
    permission: &str,
    entity: Option<Uuid>,
) -> Result<bool, OpenFabErr> {
    let mut conn = pool.acquire().await?;
    // check if the user directly has the requested permission
    let query = r#"
        SELECT 
            EXISTS (
                -- 1. Check direct user permissions
                SELECT 1 FROM user_permissions_rel 
                WHERE account = $1 
                  AND permission = $2 
                  AND (($3::uuid IS NULL AND entity IS NULL) OR (entity = $3::uuid))
            ) 
            OR 
            EXISTS (
                -- 2. Check group permissions via group memberships
                SELECT 1 FROM group_permissions_rel gp
                JOIN group_membership gm ON gp.account_group = gm.account_group
                WHERE gm.account = $1 
                  AND gp.permission = $2 
                  AND (($3::uuid IS NULL AND gp.entity IS NULL) OR (gp.entity = $3::uuid))
            ) AS allowed;
    "#;

    let row: (bool,) = sqlx::query_as(query)
        .bind(user)
        .bind(permission)
        .bind(entity)
        .fetch_one(&mut *conn)
        .await?;

    if row.0 {
        Ok(true)
    } else {
        Err(OpenFabErr::AuthErr(AuthErr::PermissionDenied(
            permission.to_string(),
        )))
    }
    /*
    let user_perms = match entity {
        Some(entity) => {
            UserPermissionRel::select()
                .where_("account = ? and permission = ? and entity = ?")
                .bind(user)
                .bind(&permission)
                .bind(entity)
                .fetch_all(&mut *conn)
                .await?
        }
        None => {
            UserPermissionRel::select()
                .where_("account = ? and permission = ?")
                .bind(user)
                .bind(&permission)
                .fetch_all(&mut *conn)
                .await?
        }
    };

    if user_perms.len() > 0 {
        return Ok(true);
    }

    let groups = GroupMembershipRel::select()
        .where_("account = ?")
        .bind(user)
        .fetch_all(&mut *conn)
        .await?
        .into_iter()
        .map(|g| g.account_group)
        .collect::<Vec<Uuid>>();
    if groups.len() < 1 {
        return Err(OpenFabErr::AuthErr(AuthErr::PermissionDenied(
            permission.to_string(),
        )));
    }

    let group_perms = match entity {
        Some(ent) => {
            GroupPermissionRel::select()
                .where_("account_group = ANY(?) and permission = ? and entity = ?")
                .bind(groups)
                .bind(permission)
                .bind(ent)
                .fetch_all(&mut *conn)
                .await?
        }
        None => {
            GroupPermissionRel::select()
                .where_("account_group = ANY(?) and permission = ?")
                .bind(groups)
                .bind(permission)
                .fetch_all(&mut *conn)
                .await?
        }
    };

    //let group_perms: Vec<String> = Vec::new();
    if group_perms.len() > 0 {
        return Ok(true);
    }
    Err(OpenFabErr::AuthErr(AuthErr::PermissionDenied(
        permission.to_string(),
    )))*/
}
