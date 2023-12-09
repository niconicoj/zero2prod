use zero2prod_macros::integration_test;

#[integration_test]
fn health_check_works(test_stack: TestStack) {
    let response = test_stack
        .client
        .health_check()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
