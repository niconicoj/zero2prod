use reqwest::StatusCode;
use zero2prod_macros::integration_test;

#[integration_test]
fn subscribe_returns_a_200_for_valid_form_data(test_app: TestApp) {
    let client = reqwest::Client::new();

    let body = "name=John%20Doe&email=john.doe@gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john.doe@gmail.com");
    assert_eq!(saved.name, "John Doe");
}

#[integration_test]
fn subscribe_returns_a_400_for_invalid_form_data() {
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=John%20Doe", "missing the email"),
        ("email=john.doe@gmail.com", "missing the name"),
        ("", "empty"),
    ];

    for (body, error) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            StatusCode::BAD_REQUEST,
            "Expected a 400 Bad Request when the payload was {} but got {} instead",
            error,
            response.status()
        );
    }
}
