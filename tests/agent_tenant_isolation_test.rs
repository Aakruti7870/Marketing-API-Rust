//! Regression tests for workspace isolation in AI agent services.
use golde_marketing_api::services::agent_service;
use sqlx::{postgres::PgPoolOptions, PgPool};
use uuid::Uuid;

async fn test_pool() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("CI must provide DATABASE_URL for isolation tests");
    PgPoolOptions::new()
        .max_connections(3)
        .connect(&database_url)
        .await
        .expect("connect to test PostgreSQL")
}

#[tokio::test]
async fn agent_services_cannot_read_update_or_delete_another_workspaces_agent() {
    let pool = test_pool().await;
    let suffix = Uuid::new_v4().simple().to_string();
    let user_id = Uuid::new_v4();
    let workspace_a = Uuid::new_v4();
    let workspace_b = Uuid::new_v4();
    let agent_a = Uuid::new_v4();
    let agent_b = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, first_name, last_name, role)
         VALUES ($1, $2, 'test-hash', 'Agent', 'Isolation', 'USER')",
    )
    .bind(user_id)
    .bind(format!("agent-isolation-{suffix}@example.test"))
    .execute(&pool)
    .await
    .expect("insert test user");

    for (workspace_id, slug) in [
        (workspace_a, format!("agent-isolation-a-{suffix}")),
        (workspace_b, format!("agent-isolation-b-{suffix}")),
    ] {
        sqlx::query("INSERT INTO workspaces (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id)
            .bind("Agent tenant isolation regression test")
            .bind(slug)
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("insert test workspace");
    }

    for (id, workspace_id, name, public_key) in [
        (
            agent_a,
            workspace_a,
            "Workspace A agent",
            format!("agent-a-{suffix}"),
        ),
        (
            agent_b,
            workspace_b,
            "Workspace B agent",
            format!("agent-b-{suffix}"),
        ),
    ] {
        sqlx::query(
            "INSERT INTO ai_agents
             (id, workspace_id, name, template_key, industry, role, description, status,
              system_prompt, created_by_id, public_key)
             VALUES ($1, $2, $3, 'SALES', 'SALES', 'Sales Agent', 'Isolation fixture',
                     'DRAFT', 'Test-only prompt', $4, $5)",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .bind(user_id)
        .bind(public_key)
        .execute(&pool)
        .await
        .expect("insert test agent");
    }

    let agents_a = agent_service::list_agents(&pool, workspace_a)
        .await
        .expect("list workspace A agents");
    assert!(agents_a.iter().any(|agent| agent.id == agent_a));
    assert!(!agents_a.iter().any(|agent| agent.id == agent_b));
    assert_eq!(agents_a.len(), 1);

    assert!(agent_service::get_agent(&pool, workspace_a, agent_b)
        .await
        .is_err());

    let update_result = agent_service::update_agent(
        &pool,
        workspace_a,
        agent_b,
        golde_marketing_api::models::UpdateAiAgentDto {
            name: Some("Unauthorized update".to_string()),
            industry: None,
            role: None,
            description: None,
            status: None,
            system_prompt: None,
            capabilities: None,
            tools: None,
            channels: None,
            settings: None,
        },
    )
    .await;
    assert!(update_result.is_err());

    assert!(agent_service::delete_agent(&pool, workspace_a, agent_b)
        .await
        .is_err());
    assert!(agent_service::get_agent(&pool, workspace_b, agent_b)
        .await
        .is_ok());

    // Workspaces cascade-delete their own agent fixtures; the user is removed last.
    sqlx::query("DELETE FROM workspaces WHERE id = ANY($1)")
        .bind(vec![workspace_a, workspace_b])
        .execute(&pool)
        .await
        .expect("remove test workspaces");
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("remove test user");
}
