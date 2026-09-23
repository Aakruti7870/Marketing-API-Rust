use crate::error::AppError;

pub fn require_workspace_roles(current_role: &str, allowed_roles: &[&str]) -> Result<(), AppError> {
    if allowed_roles.contains(&current_role) || current_role == "OWNER" {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "Insufficient role: {} requires one of: {:?}",
            current_role, allowed_roles
        )))
    }
}
