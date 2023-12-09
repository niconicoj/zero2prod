mod health_check;
mod subscribe;

pub struct Z2PClient {
    base_url: String,
    client: reqwest::Client,
}

impl Z2PClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}
