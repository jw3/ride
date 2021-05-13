use reqwest::ClientBuilder;

use crate::event::{Error, Event};

#[derive(Clone)]
pub struct HttpEventer {
    pub insecure: bool,
    pub url: String,
}

impl HttpEventer {
    pub async fn publish(&self, e: &Event) -> Result<(), Error> {
        match ClientBuilder::new()
            .danger_accept_invalid_certs(self.insecure)
            .build()
            .expect("failed to build client")
            .post(&self.url)
            .json(&e)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::HttpClientError(e)),
        }
    }
}
