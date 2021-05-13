use crate::http::HttpEventer;
use crate::mqtt::{MqttEventer, PublisherConfig};
use crate::stdout::StdoutEventer;
use serde::Serialize;

#[derive(Serialize)]
pub struct Event {
    pub id: String,
    pub x: String,
    pub y: String,
}

#[derive(Clone)]
pub enum Publisher {
    Print(StdoutEventer),
    HttpPost(HttpEventer),
    Mqtt(MqttEventer),
}

impl Publisher {
    pub async fn stdout(pretty: bool) -> Publisher {
        Self::Print(StdoutEventer { pretty })
    }
    pub async fn http(uri: &str, insecure: bool) -> Publisher {
        Self::HttpPost(HttpEventer {
            insecure,
            uri: uri.into(),
        })
    }
    pub async fn mqtt(uri: &str, topic: &str) -> Publisher {
        let mut cfg = PublisherConfig::new();
        cfg.topic = topic.into();
        cfg.uri = uri.into();
        cfg.qos = 1;

        let e = cfg.finalize().await;
        Publisher::Mqtt(e)
    }

    pub async fn publish(self, e: Event) -> Result<(), String> {
        match self {
            Publisher::Print(p) => p.publish(&e).await,
            Publisher::HttpPost(p) => p.publish(&e).await,
            Publisher::Mqtt(p) => p.publish(&e).await,
        }
    }
}
