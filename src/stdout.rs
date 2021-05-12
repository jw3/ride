use crate::event::Event;

#[derive(Clone)]
pub struct StdoutEventer {
    pub pretty: bool,
}

impl StdoutEventer {
    pub async fn publish(&self, e: &Event) -> Result<(), String> {
        let json = if self.pretty {
            serde_json::to_string_pretty(&e).unwrap()
        } else {
            serde_json::to_string(&e).unwrap()
        };
        println!("{}", json);

        Ok(())
    }
}
