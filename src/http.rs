use reqwest::ClientBuilder;

use crate::event::Event;

#[derive(Clone)]
pub struct HttpEventer {
    pub insecure: bool,
    pub uri: String,
}

impl HttpEventer {
    pub async fn publish(&self, e: &Event) -> Result<(), String> {
        match ClientBuilder::new()
            .danger_accept_invalid_certs(self.insecure)
            .build()
            .expect("failed to build client")
            .post(&self.uri)
            .json(&e)
            .send().await {
            Ok(_) => Ok(()),
            Err(_) => {
                println!("errrrrrrr");
                Err("".into())
            }
        }
    }
}
