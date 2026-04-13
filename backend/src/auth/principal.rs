use uuid::Uuid;

use crate::ports::data_scope::DataScope;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub sub: Uuid,
    pub roles: Vec<String>,
}

impl AuthUser {
    pub fn is_admin(&self) -> bool {
        self.roles.iter().any(|r| r == "admin")
    }

    pub fn data_scope(&self) -> DataScope {
        DataScope::from_auth(self.is_admin(), self.sub)
    }

    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.is_admin() || self.roles.iter().any(|r| roles.contains(&r.as_str()))
    }

    pub fn primary_role_for_audit(&self) -> &'static str {
        if self.is_admin() {
            return "admin";
        }
        if self.roles.iter().any(|r| r == "dispatcher") {
            return "dispatcher";
        }
        if self.roles.iter().any(|r| r == "supervisor") {
            return "supervisor";
        }
        if self.roles.iter().any(|r| r == "technician") {
            return "technician";
        }
        if self.roles.iter().any(|r| r == "viewer") {
            return "viewer";
        }
        "user"
    }
}
