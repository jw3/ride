use serde::Serialize;
use crate::http::HttpEventer;
use crate::stdout::StdoutEventer;

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
}

impl Publisher {
    pub fn stdout(pretty: bool) -> Publisher {
        Self::Print(StdoutEventer { pretty })
    }
    pub fn http(uri: &str, insecure: bool) -> Publisher {
        Self::HttpPost(HttpEventer{ insecure, uri: uri.into() })
    }

    pub async fn publish(self, e: Event) -> Result<(), String> {
        match self {
            Publisher::Print(p) => {
                p.publish(&e).await
            }
            Publisher::HttpPost(p) => {
                p.publish(&e).await
            }
        }
    }
}

