use uuid::Uuid;

/// Область видимости данных для репозиториев (ownership + RBAC).
#[derive(Clone, Debug)]
pub enum DataScope {
    /// Роль `admin`: все строки.
    All,
    /// Обычные роли: только строки с `owner_user_id == Owner.0`.
    Owner(Uuid),
}

impl DataScope {
    pub fn from_auth(is_admin: bool, user_id: Uuid) -> Self {
        if is_admin {
            DataScope::All
        } else {
            DataScope::Owner(user_id)
        }
    }

    pub fn is_all(&self) -> bool {
        matches!(self, DataScope::All)
    }
}
