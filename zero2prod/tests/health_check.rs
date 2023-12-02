use zero2prod_macros::integration_test;

#[integration_test]
fn health_check_works(test_stack: TestStack) {
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", test_stack.app.address))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
