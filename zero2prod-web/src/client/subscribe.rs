use super::Z2PClient;

impl Z2PClient {
    pub async fn subscribe<T>(&self, body: T) -> reqwest::Result<reqwest::Response>
    where
        T: Into<reqwest::Body>,
    {
        self.client
            .post(&format!("{}/subscriptions", self.base_url))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
    }
}
