use crate::error::AppError;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    bcrypt::hash(password, 10).map_err(|e| AppError::Internal(anyhow::anyhow!("Password hash error: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(password, hash).map_err(|e| AppError::Internal(anyhow::anyhow!("Password verify error: {}", e)))
}
