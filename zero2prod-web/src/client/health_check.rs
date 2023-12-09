use super::Z2PClient;

impl Z2PClient {
    pub async fn health_check(&self) -> reqwest::Result<reqwest::Response> {
        self.client
            .get(&format!("{}/health_check", self.base_url))
            .send()
            .await
    }
}
