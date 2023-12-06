use crate::helpers::{assert_is_redirect_to, spawn_app};
use uuid::Uuid;

#[tokio::test]
async fn you_must_login_to_see_change_password_form() {
    // Arrange
    let app = spawn_app().await;
    // Act
    let response = app.get_change_password().await;
    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn you_must_login_to_change_password() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    // Act
    let response = app.post_change_password(&serde_json::json!({"current_password": Uuid::new_v4().to_string(),"new_password": &new_password, "new_password_check": &new_password})).await;
    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Act 1 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;

    // Act 2 - Change Password
    let response = app.post_change_password(&serde_json::json!({"current_password": &app.test_user.password,"new_password": &new_password, "new_password_check": &another_new_password})).await;
    assert_is_redirect_to(&response, "/admin/password");
    // Act 3 - Follow redirect
    let html_page = app.get_change_password_html().await;
    assert!(
        html_page.contains("You entered two different new passwords - the field values must match")
    );
}

#[tokio::test]
async fn current_password_must_be_valid() {
    //Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();
    // Act 1 - Login
    app.post_login(&serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password
    }))
    .await;
    // Act 2 - Change Password
    let response = app.post_change_password(&serde_json::json!({"current_password": &wrong_password,"new_password": &new_password, "new_password_check": &new_password})).await;
    assert_is_redirect_to(&response, "/admin/password");
    // Assert
    let html_page = app.get_change_password_html().await;
    assert!(html_page.contains("The current password is incorrect."));
}
