use serde_json::json;

#[test]
fn test_agent_approval_payload() {
    let approval_payload = json!({
        "status": "WAITING_APPROVAL",
        "requires_approval": true,
        "input_payload": {
            "recipientsCount": 14,
            "discountTier": "14%"
        }
    });

    assert_eq!(approval_payload["status"], "WAITING_APPROVAL");
    assert_eq!(approval_payload["requires_approval"], true);
    assert_eq!(approval_payload["input_payload"]["recipientsCount"], 14);
}
