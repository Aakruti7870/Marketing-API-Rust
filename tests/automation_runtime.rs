use golde_marketing_api::models::CreateAutomationDto;
use golde_marketing_api::services::automation_service;
use reqwest::Client;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

async fn test_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("database connection")
}

async fn create_test_tenant(pool: &sqlx::PgPool) -> (Uuid, Uuid) {
    let user_id = Uuid::new_v4();
    let workspace_id = Uuid::new_v4();
    let email = format!("automation-runtime-{}@example.invalid", user_id);

    sqlx::query(
        "INSERT INTO users (id,email,password_hash,first_name,last_name,role)
         VALUES ($1,$2,$3,$4,$5,'USER')",
    )
    .bind(user_id)
    .bind(email)
    .bind("test-only")
    .bind("Runtime")
    .bind("Verification")
    .execute(pool)
    .await
    .expect("insert test user");

    sqlx::query(
        "INSERT INTO workspaces (id,name,slug,description,owner_id)
         VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(workspace_id)
    .bind("__automation_runtime_test__")
    .bind(format!("automation-runtime-{}", user_id))
    .bind("integration test")
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert test workspace");

    sqlx::query(
        "INSERT INTO workspace_members (id,workspace_id,user_id,role)
         VALUES ($1,$2,$3,'OWNER')",
    )
    .bind(Uuid::new_v4())
    .bind(workspace_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("insert test workspace member");

    (user_id, workspace_id)
}

async fn cleanup(pool: &sqlx::PgPool, user_id: Uuid) {
    let workspace_ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM workspaces WHERE owner_id=$1 AND name='__automation_runtime_test__'",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .expect("find test workspaces");

    for workspace_id in workspace_ids {
        sqlx::query("DELETE FROM workspaces WHERE id=$1")
            .bind(workspace_id)
            .execute(pool)
            .await
            .expect("cleanup test workspace");
    }

    sqlx::query("DELETE FROM users WHERE id=$1")
        .bind(user_id)
        .execute(pool)
        .await
        .expect("cleanup test user");
}

fn manual_graph() -> golde_marketing_api::models::AutomationGraph {
    serde_json::from_value(json!({
        "nodes": [
            {"id":"trigger","name":"Manual","type":"trigger.manual","config":{},"position":[0,0]},
            {"id":"set","name":"Set","type":"data.set","config":{"data":{"verified":true,"source":"integration-test"}},"position":[200,0]}
        ],
        "edges":[{"source":"trigger","target":"set"}]
    }))
    .expect("valid graph")
}

#[tokio::test]
async fn automation_runtime_persists_run_and_steps() {
    let pool = test_pool().await;
    let (user_id, workspace_id) = create_test_tenant(&pool).await;
    let http = Client::new();

    let created = automation_service::create(
        &pool,
        workspace_id,
        user_id,
        CreateAutomationDto {
            name: "__automation_runtime_test__".into(),
            description: Some("database-backed runtime test".into()),
            trigger_type: "MANUAL".into(),
            trigger_config: Some(json!({})),
            graph: manual_graph(),
        },
    )
    .await
    .expect("create automation");

    let run = automation_service::run(
        &pool,
        &http,
        workspace_id,
        created.id,
        json!({"input":"runtime"}),
    )
    .await
    .expect("execute automation");

    assert_eq!(run.status, "COMPLETED");
    assert_eq!(run.output_data["verified"], true);
    assert_eq!(run.output_data["source"], "integration-test");

    let runs = automation_service::runs(&pool, workspace_id, created.id)
        .await
        .expect("list runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, run.id);

    let steps = automation_service::run_steps(&pool, workspace_id, created.id, run.id)
        .await
        .expect("list run steps");
    assert_eq!(steps.len(), 2);
    assert!(steps.iter().all(|s| s.status == "COMPLETED"));

    let cross_workspace = automation_service::list(&pool, Uuid::new_v4())
        .await
        .expect("list another workspace");
    assert!(cross_workspace.is_empty());

    cleanup(&pool, user_id).await;
}

#[tokio::test]
async fn automation_scheduler_executes_due_schedule() {
    let pool = test_pool().await;
    let (user_id, workspace_id) = create_test_tenant(&pool).await;
    let http = Client::new();

    let mut graph = manual_graph();
    graph.nodes[0].node_type = "trigger.schedule".into();

    let created = automation_service::create(
        &pool,
        workspace_id,
        user_id,
        CreateAutomationDto {
            name: "__automation_scheduler_test__".into(),
            description: Some("scheduler integration test".into()),
            trigger_type: "SCHEDULE".into(),
            trigger_config: Some(json!({"interval_seconds":1})),
            graph,
        },
    )
    .await
    .expect("create scheduled automation");

    automation_service::set_status(&pool, workspace_id, created.id, "ACTIVE")
        .await
        .expect("activate scheduled automation");

    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    let count = automation_service::run_due_schedules(&pool, &http)
        .await
        .expect("run due schedules");
    assert_eq!(count, 1);

    let runs = automation_service::runs(&pool, workspace_id, created.id)
        .await
        .expect("list scheduled runs");
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].status, "COMPLETED");

    let refreshed = automation_service::get(&pool, workspace_id, created.id)
        .await
        .expect("get scheduled automation");
    assert!(refreshed.last_run_at.is_some());
    assert!(refreshed.next_run_at.is_some());

    cleanup(&pool, user_id).await;
}
