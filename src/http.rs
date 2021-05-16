use reqwest::{Client, ClientBuilder};

use crate::event::Error::HttpClientError;
use crate::event::{Error, Event};

#[derive(Clone)]
pub struct HttpEmitter {
    client: Client,
    pub url: String,
}

impl HttpEmitter {
    pub async fn publish(&self, e: &Event) -> Result<(), Error> {
        match self.client.post(&self.url).json(&e).send().await {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::HttpClientError(e)),
        }
    }
}

#[derive(Default)]
pub struct Builder {
    insecure: bool,
    url: String,
}

impl Builder {
    pub fn with_insecure(&mut self, insecure: bool) -> &mut Self {
        self.insecure = insecure;
        self
    }

    pub fn with_url(&mut self, url: &str) -> &mut Self {
        self.url = url.into();
        self
    }

    pub async fn finalize(&self) -> Result<HttpEmitter, Error> {
        match ClientBuilder::new()
            .danger_accept_invalid_certs(self.insecure)
            .build()
        {
            Ok(client) => Ok(HttpEmitter {
                client,
                url: self.url.clone(),
            }),
            Err(e) => Err(HttpClientError(e)),
        }
    }
}
