use reqwest::ClientBuilder;

use crate::event::Error::HttpClientError;
use crate::event::{Error, Event};

#[derive(Clone)]
pub struct HttpEmitter {
    pub insecure: bool,
    pub url: String,
}

impl HttpEmitter {
    pub async fn publish(&self, e: &Event) -> Result<(), Error> {
        let client = ClientBuilder::new()
            .danger_accept_invalid_certs(self.insecure)
            .build()
            .map_err(HttpClientError)
            .unwrap();

        match client.post(&self.url).json(&e).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::HttpClientError(e)),
        }
    }
}
