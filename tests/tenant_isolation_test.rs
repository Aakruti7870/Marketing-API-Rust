//! Regression tests for cross-workspace data isolation.
//!
//! These tests use the migrated CI PostgreSQL database and create uniquely named
//! fixtures. They intentionally call the same service functions used by the API.

use golde_marketing_api::{
    services::{campaign_service, contact_service},
    utils::pagination::PaginationQuery,
};
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
async fn workspace_scoped_contact_and_campaign_services_reject_cross_tenant_access() {
    let pool = test_pool().await;
    let suffix = Uuid::new_v4().simple().to_string();
    let user_id = Uuid::new_v4();
    let workspace_a = Uuid::new_v4();
    let workspace_b = Uuid::new_v4();
    let contact_a = Uuid::new_v4();
    let contact_b = Uuid::new_v4();
    let campaign_a = Uuid::new_v4();
    let campaign_b = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, first_name, last_name, role)
         VALUES ($1, $2, 'test-hash', 'Isolation', 'Test', 'USER')",
    )
    .bind(user_id)
    .bind(format!("isolation-{suffix}@example.test"))
    .execute(&pool)
    .await
    .expect("insert test user");

    for (workspace_id, slug) in [
        (workspace_a, format!("isolation-a-{suffix}")),
        (workspace_b, format!("isolation-b-{suffix}")),
    ] {
        sqlx::query("INSERT INTO workspaces (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(workspace_id)
            .bind("Tenant isolation regression test")
            .bind(slug)
            .bind(user_id)
            .execute(&pool)
            .await
            .expect("insert test workspace");
    }

    for (id, workspace_id, phone) in [
        (contact_a, workspace_a, "+15550000001"),
        (contact_b, workspace_b, "+15550000002"),
    ] {
        sqlx::query(
            "INSERT INTO contacts (id, workspace_id, first_name, phone)
             VALUES ($1, $2, 'Isolation contact', $3)",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(phone)
        .execute(&pool)
        .await
        .expect("insert test contact");
    }

    for (id, workspace_id, name) in [
        (campaign_a, workspace_a, "Tenant A campaign"),
        (campaign_b, workspace_b, "Tenant B campaign"),
    ] {
        sqlx::query(
            "INSERT INTO campaigns (id, workspace_id, name, created_by_id)
             VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(name)
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("insert test campaign");
    }

    // Lists must include only records belonging to the requested workspace.
    let (contacts, contact_meta) = contact_service::list_contacts(
        &pool,
        workspace_a,
        PaginationQuery {
            page: Some(1),
            limit: Some(100),
        },
        None,
    )
    .await
    .expect("list workspace A contacts");
    assert!(contacts.iter().any(|contact| contact.id == contact_a));
    assert!(!contacts.iter().any(|contact| contact.id == contact_b));
    assert_eq!(contact_meta.total, 1);

    let (campaigns, campaign_meta) = campaign_service::list_campaigns(
        &pool,
        workspace_a,
        PaginationQuery {
            page: Some(1),
            limit: Some(100),
        },
    )
    .await
    .expect("list workspace A campaigns");
    assert!(campaigns.iter().any(|campaign| campaign.id == campaign_a));
    assert!(!campaigns.iter().any(|campaign| campaign.id == campaign_b));
    assert_eq!(campaign_meta.total, 1);

    // A known foreign ID must behave as not found, not reveal another tenant's row.
    assert!(contact_service::get_contact(&pool, workspace_a, contact_b)
        .await
        .is_err());
    assert!(
        campaign_service::get_campaign(&pool, workspace_a, campaign_b)
            .await
            .is_err()
    );

    // A cross-tenant delete must fail and leave the owner's record intact.
    assert!(
        contact_service::delete_contact(&pool, workspace_a, contact_b)
            .await
            .is_err()
    );
    assert!(contact_service::get_contact(&pool, workspace_b, contact_b)
        .await
        .is_ok());

    // Remove only fixtures created by this test.
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
