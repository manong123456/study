//! Integration tests for the error handling system.

use capeos_common::error::{AppError, AppResult};

#[test]
fn test_error_display_messages() {
    let cases: Vec<(AppError, &str)> = vec![
        (AppError::NotFound("item".to_string()), "not found: item"),
        (
            AppError::Unauthorized("bad token".to_string()),
            "unauthorized: bad token",
        ),
        (
            AppError::BadRequest("missing field".to_string()),
            "bad request: missing field",
        ),
        (
            AppError::Internal("db error".to_string()),
            "internal: db error",
        ),
    ];
    for (err, expected) in cases {
        assert_eq!(format!("{}", err), expected);
    }
}

#[test]
fn test_app_result_ok() {
    let result: AppResult<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_app_result_err() {
    let result: AppResult<i32> = Err(AppError::NotFound("test".to_string()));
    assert!(result.is_err());
}

#[test]
fn test_anyhow_conversion() {
    let anyhow_err = anyhow::anyhow!("something went wrong");
    let app_err: AppError = anyhow_err.into();
    assert!(format!("{}", app_err).contains("something went wrong"));
}
