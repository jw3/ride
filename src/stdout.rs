use crate::event::Error::JsonError;
use crate::event::{Error, Event};

#[derive(Clone)]
pub struct StdoutEmitter {
    pub pretty: bool,
}

impl StdoutEmitter {
    pub async fn publish(&self, e: &Event) -> Result<(), Error> {
        let res = if self.pretty {
            serde_json::to_string_pretty(&e)
        } else {
            serde_json::to_string(&e)
        };

        match res {
            Ok(json) => {
                println!("{}", json);
                Ok(())
            }
            Err(e) => Err(JsonError(e)),
        }
    }
}
