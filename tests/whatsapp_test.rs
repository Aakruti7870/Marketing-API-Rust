use golde_marketing_api::config::Config;
use golde_marketing_api::services::whatsapp_service::send_whatsapp_message;
use reqwest::Client;

#[tokio::test]
async fn test_whatsapp_simulation_mode_fallback() {
    let mut config = Config::from_env().unwrap();
    config.whatsapp_simulation_mode = true;
    let client = Client::new();

    let result = send_whatsapp_message(
        &client,
        &config,
        "+919822011223",
        "Test simulation message from GrowthOS",
        None,
    )
    .await
    .expect("Simulation call failed");

    assert!(result.simulated);
    assert_eq!(result.status, "SENT");
    assert!(result.message_id.starts_with("wamid.HBgL"));
}
