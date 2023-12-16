use crate::helpers::{assert_is_redirect_to, spawn_app};

#[tokio::test]
async fn you_must_log_in_to_access_admin_dashboard() {
    // Arrange
    let app = spawn_app().await;
    // Act
    let response = app.get_admin_dashboard().await;
    // Assert
    assert_is_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    // Arrange
    let app = spawn_app().await;
    // Act 1 - Login
    let response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password
        }))
        .await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act 2 - Follow redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));

    // Act 3 - logout
    let response = app.post_logout().await;
    assert_is_redirect_to(&response, "/login");

    // Act 4 - Follow redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains(r#"You have successfully logged out."#));

    // Act 5 - Attempt Load admin panel
    let response = app.get_admin_dashboard().await;
    assert_is_redirect_to(&response, "/login");
}
