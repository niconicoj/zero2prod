use zero2prod_core::testing::run_test;

#[test]
fn health_check_works() {
    run_test(|test_app| {
        Box::pin(async move {
            let client = reqwest::Client::new();

            let response = client
                .get(&format!("{}/health_check", test_app.app_address))
                .send()
                .await
                .expect("Failed to execute request");

            assert!(response.status().is_success());
            assert_eq!(Some(0), response.content_length());
        })
    })
}
