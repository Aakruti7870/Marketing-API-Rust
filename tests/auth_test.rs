use golde_marketing_api::auth::{hash_password, verify_password, generate_access_token, verify_token};
use golde_marketing_api::config::Config;
use uuid::Uuid;

#[test]
fn test_password_hashing() {
    let password = "SuperSecretPassword123!";
    let hash = hash_password(password).expect("Hashing failed");
    assert!(verify_password(password, &hash).expect("Verification failed"));
    assert!(!verify_password("WrongPassword!", &hash).expect("Verification failed"));
}

#[test]
fn test_jwt_token_generation_and_verification() {
    let config = Config::from_env().unwrap();
    let user_id = Uuid::new_v4();
    let email = "tester@golde-ai.com";
    let role = "USER";
    let workspace_id = Some(Uuid::new_v4());

    let token = generate_access_token(user_id, email, role, workspace_id, &config)
        .expect("JWT generation failed");

    let claims = verify_token(&token, &config.jwt_access_secret)
        .expect("JWT verification failed");

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
    assert_eq!(claims.role, role);
    assert_eq!(claims.workspace_id, workspace_id);
}
