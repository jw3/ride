use paho_mqtt as mqtt;
use paho_mqtt::{AsyncClient, CreateOptionsBuilder};

use crate::event::Error::MqttConnectError;
use crate::event::{Error, Event};

#[derive(Default)]
pub struct EmitterConfig {
    uri: String,
    topic: String,
    qos: i32,
}

impl EmitterConfig {
    pub fn with_uri(&mut self, uri: &str) -> &mut Self {
        self.uri = uri.into();
        self
    }

    pub fn with_topic(&mut self, topic: &str) -> &mut Self {
        self.topic = topic.into();
        self
    }

    pub fn with_qos(&mut self, qos: i32) -> &mut Self {
        self.qos = qos;
        self
    }

    pub async fn finalize(&self) -> Result<MqttEmitter, Error> {
        let opts = CreateOptionsBuilder::default()
            .server_uri(&self.uri)
            .finalize();

        let conn_opts = mqtt::ConnectOptions::new();
        let cli = mqtt::async_client::AsyncClient::new(opts).expect("bad client");
        match cli.connect(conn_opts).await {
            Err(e) => Err(MqttConnectError(e)),
            Ok(_) => Ok(MqttEmitter {
                cli,
                topic: self.topic.clone(),
                qos: self.qos,
            }),
        }
    }
}

#[derive(Clone)]
pub struct MqttEmitter {
    pub cli: AsyncClient,
    pub topic: String,
    pub qos: i32,
}

impl MqttEmitter {
    pub async fn publish(&self, e: &Event) -> Result<(), Error> {
        let topic = mqtt::Topic::new(&self.cli, &self.topic, self.qos);
        let m = serde_json::to_string(&e).unwrap();
        match topic.publish(m).await {
            Ok(_) => Ok(()),
            Err(e) => Err(MqttConnectError(e)),
        }
    }
}
