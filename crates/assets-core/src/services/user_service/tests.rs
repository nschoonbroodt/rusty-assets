use super::*;
use crate::tests::utils::*;

#[tokio::test]
async fn test_create_user() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool);

    let new_user = NewUser::builder()
        .name("testuser")
        .display_name("Test User")
        .build();
    let result = service.create_user(new_user).await;

    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "testuser");
    assert_eq!(user.display_name, "Test User");
    assert!(user.is_active);
}

#[tokio::test]
async fn test_create_user_duplicate_name() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool);

    // Create first user
    let new_user1 = NewUser::builder()
        .name("duplicate")
        .display_name("First User")
        .build();
    assert!(service.create_user(new_user1).await.is_ok());

    // Try to create user with same name
    let new_user2 = NewUser::builder()
        .name("duplicate")
        .display_name("Second User")
        .build();
    let result = service.create_user(new_user2).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("already exists"));
}

#[tokio::test]
async fn test_create_user_validation_errors() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool);

    // Test empty name
    let new_user = NewUser::builder()
        .name("")
        .display_name("Test User")
        .build();
    assert!(service.create_user(new_user).await.is_err());

    // Test name too long
    let long_name = "a".repeat(51);
    let new_user = NewUser::builder()
        .name(long_name)
        .display_name("Test User")
        .build();
    assert!(service.create_user(new_user).await.is_err());

    // Test display name too long
    let long_display_name = "a".repeat(101);
    let new_user = NewUser::builder()
        .name("testuser")
        .display_name(long_display_name)
        .build();
    assert!(service.create_user(new_user).await.is_err());

    // Test empty display name
    let new_user = NewUser::builder().name("testuser").display_name("").build();
    assert!(service.create_user(new_user).await.is_err());
}

#[tokio::test]
async fn test_get_user_by_id() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let created_user = create_test_user_with_names(&pool, "testuser", "Test User").await;

    // Test existing user
    let result = service.get_user(created_user.id).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert!(user.is_some());
    let user = user.unwrap();
    assert_eq!(user.id, created_user.id);
    assert_eq!(user.name, "testuser");
    assert_eq!(user.display_name, "Test User");

    // Test non-existent user
    let non_existent_id = Uuid::new_v4();
    let result = service.get_user(non_existent_id).await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[tokio::test]
async fn test_update_user() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let created_user = create_test_user_with_names(&pool, "testuser", "Test User").await;

    // Test valid update
    let updates = UserUpdates {
        display_name: Some("Updated Display Name".to_string()),
    };
    let result = service.update_user(created_user.id, updates).await;
    assert!(result.is_ok());
    let updated_user = result.unwrap();
    assert_eq!(updated_user.display_name, "Updated Display Name");
    assert_eq!(updated_user.name, "testuser"); // Name unchanged

    // Test empty updates
    let empty_updates = UserUpdates::default();
    let result = service.update_user(created_user.id, empty_updates).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No updates provided")
    );

    // Test non-existent user
    let non_existent_id = Uuid::new_v4();
    let updates = UserUpdates {
        display_name: Some("New Name".to_string()),
    };
    let result = service.update_user(non_existent_id, updates).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_update_user_validation() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let created_user = create_test_user_with_names(&pool, "testuser", "Test User").await;

    // Test display name too long
    let long_display_name = "a".repeat(101);
    let updates = UserUpdates {
        display_name: Some(long_display_name),
    };
    let result = service.update_user(created_user.id, updates).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("too long"));

    // Test empty display name
    let updates = UserUpdates {
        display_name: Some("".to_string()),
    };
    let result = service.update_user(created_user.id, updates).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cannot be empty"));
}

#[tokio::test]
async fn test_deactivate_user() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let created_user = create_test_user_with_names(&pool, "testuser", "Test User").await;

    // Test successful deactivation
    let result = service.deactivate_user(created_user.id).await;
    assert!(result.is_ok());

    // Verify user is deactivated
    let user = service.get_user(created_user.id).await.unwrap().unwrap();
    assert!(!user.is_active);

    // Test deactivating already inactive user
    let result = service.deactivate_user(created_user.id).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("not found or already inactive")
    );

    // Test non-existent user
    let non_existent_id = Uuid::new_v4();
    let result = service.deactivate_user(non_existent_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_reactivate_user() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let created_user = create_test_user_with_names(&pool, "testuser", "Test User").await;

    // First deactivate the user
    service.deactivate_user(created_user.id).await.unwrap();

    // Test successful reactivation
    let result = service.reactivate_user(created_user.id).await;
    assert!(result.is_ok());
    let reactivated_user = result.unwrap();
    assert!(reactivated_user.is_active);
    assert_eq!(reactivated_user.name, "testuser");

    // Test reactivating already active user
    let result = service.reactivate_user(created_user.id).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("not found or already active")
    );

    // Test non-existent user
    let non_existent_id = Uuid::new_v4();
    let result = service.reactivate_user(non_existent_id).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_all_users() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let user1 = create_test_user_with_names(&pool, "user1", "User One").await;
    let user2 = create_test_user_with_names(&pool, "user2", "User Two").await;

    let result = service.get_all_users().await;
    assert!(result.is_ok());
    let users = result.unwrap();
    assert_eq!(users.len(), 2);
    assert!(users.iter().any(|u| u.id == user1.id));
    assert!(users.iter().any(|u| u.id == user2.id));
    assert!(users.iter().all(|u| u.is_active));
}

#[tokio::test]
async fn test_get_all_users_with_filter() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let user1 = create_test_user_with_names(&pool, "user1", "User One").await;
    let user2 = create_test_user_with_names(&pool, "user2", "User Two").await;

    // Deactivate one user
    service.deactivate_user(user2.id).await.unwrap();

    // Test active users only (default behavior)
    let active_users = service.get_all_users_with_filter(false).await.unwrap();
    assert_eq!(active_users.len(), 1);
    assert_eq!(active_users[0].id, user1.id);

    // Test all users including inactive
    let all_users = service.get_all_users_with_filter(true).await.unwrap();
    assert_eq!(all_users.len(), 2);
    assert!(all_users.iter().any(|u| u.id == user1.id && u.is_active));
    assert!(all_users.iter().any(|u| u.id == user2.id && !u.is_active));
}

#[tokio::test]
async fn test_get_user_by_name() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let user1 = create_test_user_with_names(&pool, "user1", "User One").await;
    let _user2 = create_test_user_with_names(&pool, "user2", "User Two").await;

    // Test existing user
    let user = service.get_user_by_name("user1").await;
    assert!(user.is_ok());
    let user = user.unwrap();
    assert!(user.is_some());
    assert_eq!(user.unwrap().id, user1.id);

    // Test non-existent user
    let user = service.get_user_by_name("nonexistent").await;
    assert!(user.is_ok());
    assert!(user.unwrap().is_none());
}

#[tokio::test]
async fn test_get_first_user() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool.clone());

    let _user1 = create_test_user_with_names(&pool, "user1", "User One").await;
    let _user2 = create_test_user_with_names(&pool, "user2", "User Two").await;

    let user = service.get_first_user().await;
    assert!(user.is_ok());
    let user = user.unwrap();
    assert!(user.is_some());
    assert_eq!(user.unwrap().name, "user1"); // First created user
}

#[tokio::test]
async fn test_get_first_user_empty_db() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool);

    let user = service.get_first_user().await;
    assert!(user.is_ok());
    assert!(user.unwrap().is_none());
}

#[tokio::test]
async fn test_unicode_and_special_characters() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool);

    // Test Unicode characters
    let new_user = NewUser::builder()
        .name("用户名")
        .display_name("用户显示名称 🎉")
        .build();
    let result = service.create_user(new_user).await;
    assert!(result.is_ok());
    let user = result.unwrap();
    assert_eq!(user.name, "用户名");
    assert_eq!(user.display_name, "用户显示名称 🎉");
}

#[tokio::test]
async fn test_user_lifecycle_integration() {
    let (pool, _container) = setup_test_db().await;
    let service = UserService::new(pool);

    // Create user
    let new_user = NewUser::builder()
        .name("lifecycle_user")
        .display_name("Lifecycle Test User")
        .build();
    let created_user = service.create_user(new_user).await.unwrap();

    // Update user
    let updates = UserUpdates {
        display_name: Some("Updated Lifecycle User".to_string()),
    };
    let updated_user = service.update_user(created_user.id, updates).await.unwrap();
    assert_eq!(updated_user.display_name, "Updated Lifecycle User");

    // Deactivate user
    service.deactivate_user(created_user.id).await.unwrap();

    // Verify not in active list
    let active_users = service.get_all_users().await.unwrap();
    assert!(!active_users.iter().any(|u| u.id == created_user.id));

    // Reactivate user
    let reactivated_user = service.reactivate_user(created_user.id).await.unwrap();
    assert!(reactivated_user.is_active);
    assert_eq!(reactivated_user.display_name, "Updated Lifecycle User");

    // Verify back in active list
    let active_users = service.get_all_users().await.unwrap();
    assert!(active_users.iter().any(|u| u.id == created_user.id));
}
