use crate::models::{AccountSubtype, AccountType, NewAccount, NewUser, User};
use crate::services::UserService;
use sqlx::PgPool;
use testcontainers::{ContainerAsync, GenericImage, ImageExt, runners::AsyncRunner};

/// Test helper to create a test database with migrations
/// Returns (pool, container) - keep container alive for the test duration
pub async fn setup_test_db() -> (PgPool, ContainerAsync<GenericImage>) {
    // Start PostgreSQL in a container
    let postgres = GenericImage::new("postgres", "15")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "test_db")
        .with_env_var("POSTGRES_USER", "postgres")
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    let port = postgres
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get PostgreSQL port");

    let database_url = format!(
        "postgres://postgres:password@localhost:{}/test_db?sslmode=disable",
        port
    );

    // Wait a bit for PostgreSQL to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Connect to the database with retries
    let mut attempts = 0;
    let pool = loop {
        match PgPool::connect(&database_url).await {
            Ok(pool) => break pool,
            Err(e) if attempts < 5 => {
                attempts += 1;
                eprintln!("Connection attempt {} failed: {}. Retrying...", attempts, e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                continue;
            }
            Err(e) => panic!("Failed to connect to test database after 5 attempts: {}", e),
        }
    };

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, postgres)
}

/// Test helper to create a test user with default name and display name
pub async fn create_test_user(pool: &PgPool) -> User {
    create_test_user_with_names(pool, "test_user", "Test User").await
}

/// Test helper to create a test user with custom names
pub async fn create_test_user_with_names(pool: &PgPool, name: &str, display_name: &str) -> User {
    let user_service = UserService::new(pool.clone());
    user_service
        .create_user(
            NewUser::builder()
                .name(name)
                .display_name(display_name)
                .build(),
        )
        .await
        .expect("Failed to create test user")
}

/// Test helper to create a basic NewAccount with Asset/Checking defaults
pub fn create_test_new_account() -> NewAccount {
    create_test_new_account_with_name("Test Account")
}

/// Test helper to create a basic NewAccount with custom name
pub fn create_test_new_account_with_name(name: &str) -> NewAccount {
    NewAccount {
        name: name.to_string(),
        account_type: AccountType::Asset,
        account_subtype: AccountSubtype::Checking,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    }
}

/// Test helper to create a NewAccount with custom type and subtype
pub fn create_test_new_account_with_type(
    name: &str,
    account_type: AccountType,
    account_subtype: AccountSubtype,
) -> NewAccount {
    NewAccount {
        name: name.to_string(),
        account_type,
        account_subtype,
        parent_id: None,
        currency: "EUR".to_string(),
        symbol: None,
        quantity: None,
        average_cost: None,
        address: None,
        purchase_date: None,
        purchase_price: None,
        notes: None,
    }
}
