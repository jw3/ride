use crate::http;
use crate::mqtt;
use crate::stdout::StdoutEmitter;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MQTT error: {0}")]
    MqttConnectError(#[from] paho_mqtt::Error),
    #[error("HTTP error: {0}")]
    HttpClientError(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

#[derive(Serialize)]
pub struct Event {
    pub id: String,
    pub x: String,
    pub y: String,
    pub spd: String,
}

#[derive(Clone)]
pub enum Publisher {
    Print(StdoutEmitter),
    HttpPost(http::HttpEmitter),
    Mqtt(mqtt::MqttEmitter),
}

impl Publisher {
    pub async fn stdout(pretty: bool) -> Result<Publisher, Error> {
        Ok(Self::Print(StdoutEmitter { pretty }))
    }
    pub async fn http(url: &str, insecure: bool) -> Result<Publisher, Error> {
        http::Builder::default()
            .with_url(url)
            .with_insecure(insecure)
            .finalize()
            .await
            .map(Publisher::HttpPost)
    }
    pub async fn mqtt(uri: &str, topic: &str) -> Result<Publisher, Error> {
        mqtt::Builder::default()
            .with_uri(uri)
            .with_topic(topic)
            .finalize()
            .await
            .map(Publisher::Mqtt)
    }

    pub async fn publish(self, e: Event) -> Result<(), Error> {
        match self {
            Publisher::Print(p) => p.publish(&e).await,
            Publisher::HttpPost(p) => p.publish(&e).await,
            Publisher::Mqtt(p) => p.publish(&e).await,
        }
    }
}
