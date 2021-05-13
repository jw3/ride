use paho_mqtt as mqtt;
use paho_mqtt::{AsyncClient, CreateOptionsBuilder};

use crate::event::Event;

pub struct PublisherConfig {
    pub uri: String,
    pub topic: String,
    pub qos: i32,
}

impl PublisherConfig {
    pub fn new() -> Self {
        PublisherConfig {
            uri: "".to_string(),
            topic: "".to_string(),
            qos: 0,
        }
    }

    pub async fn finalize(&self) -> MqttEventer {
        let opts = CreateOptionsBuilder::default()
            .server_uri(&self.uri)
            .finalize();

        let conn_opts = mqtt::ConnectOptions::new();

        let cli = mqtt::async_client::AsyncClient::new(opts).expect("bad client");
        cli.connect(conn_opts).await.expect("client connect");

        MqttEventer {
            cli,
            topic: self.topic.clone(),
            qos: self.qos,
        }
    }
}

#[derive(Clone)]
pub struct MqttEventer {
    pub cli: AsyncClient,
    pub topic: String,
    pub qos: i32,
}

impl MqttEventer {
    pub async fn publish(&self, e: &Event) -> Result<(), String> {
        let topic = mqtt::Topic::new(&self.cli, &self.topic, self.qos);
        let m = serde_json::to_string(&e).unwrap();
        topic.publish(m).await.expect("failed to publish");

        Ok(())
    }
}
