use reqwest::StatusCode;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};
use zero2prod_macros::integration_test;

#[integration_test]
fn subscribe_returns_a_200_for_valid_form_data(test_stack: TestStack) {
    let body = "name=John%20Doe&email=john.doe@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_stack.email_server)
        .await;

    let response = test_stack
        .client
        .subscribe(body)
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_stack.app.pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "john.doe@gmail.com");
    assert_eq!(saved.name, "John Doe");
}

#[integration_test]
fn subscribe_sends_a_confirmation_email_for_valid_data(test_stack: TestStack) {
    let body = "name=John%20Doe&email=john.doe@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_stack.email_server)
        .await;

    let response = test_stack
        .client
        .subscribe(body)
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let email_request = test_stack
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    let body: serde_json::Value = email_request.body_json().unwrap();
    let expected_link = &format!(
        "http://{}/subscriptions/confirm",
        test_stack.app.config.app.host
    );
    println!("Could not find {} in {}", expected_link, body);
    assert!(body.to_string().contains(expected_link));
}

#[integration_test]
fn subscribe_returns_a_400_for_invalid_form_data(test_stack: TestStack) {
    let test_cases = vec![
        ("name=John%20Doe", "missing the email"),
        ("email=john.doe@gmail.com", "missing the name"),
        ("", "empty"),
        ("name=John%20Doe&email=hello.world", "invalid email"),
    ];

    for (body, error) in test_cases {
        let response = test_stack
            .client
            .subscribe(body)
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
