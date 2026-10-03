use keyring::Entry;
use crate::error::AppError;

pub const SERVICE_NAME: &str = "dev.fugdb.credentials";

pub fn set_secret(key: &str, secret: &str) -> Result<(), AppError> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| AppError::KeyringError(format!("Failed to initialize keyring entry: {}", e)))?;
    
    entry.set_password(secret)
        .map_err(|e| AppError::KeyringError(format!("Failed to save credential to OS keyring: {}", e)))
}

pub fn get_secret(key: &str) -> Result<Option<String>, AppError> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| AppError::KeyringError(format!("Failed to initialize keyring entry: {}", e)))?;
    
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::KeyringError(format!("Failed to retrieve credential from OS keyring: {}", e))),
    }
}

pub fn delete_secret(key: &str) -> Result<bool, AppError> {
    let entry = Entry::new(SERVICE_NAME, key)
        .map_err(|e| AppError::KeyringError(format!("Failed to initialize keyring entry: {}", e)))?;
    
    match entry.delete_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(AppError::KeyringError(format!("Failed to delete credential from OS keyring: {}", e))),
    }
}
