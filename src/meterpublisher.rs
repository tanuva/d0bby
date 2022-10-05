use std::time::Duration;
use log::error;
use crate::metervalues::MeterValues;
use paho_mqtt as mqtt;

pub struct MeterPublisher {
    client: mqtt::Client,
}

impl MeterPublisher {
    pub fn new(url: &str) -> Result<MeterPublisher, String> {
        // tcp://idefix.local:1883
        let client = match mqtt::Client::new(url) {
            Ok(client) => client,
            Err(err) => {
                let msg = format!("{:?}", err);
                return Err(msg);
            },
        };

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            .finalize();

        if let Err(err) = client.connect(conn_opts) {
            let msg = format!("Cannot connect to {}: {:?}", url, err);
            return Err(msg);
        }

        Ok(MeterPublisher {
            client,
        })
    }

    pub fn publish(&self, values: &MeterValues) -> Result<(), String> {
        if !self.client.is_connected() {
            if let Err(err) = self.client.reconnect() {
                let msg = format!("{:?}", err);
                return Err(msg);
            }
        }

        let msg = mqtt::Message::new("test", "Hello world!", 0);
        if let Err(err) = self.client.publish(msg) {
            let msg = format!("{:?}", err);
            return Err(msg);
        }

        Ok(())
    }
}

impl Drop for MeterPublisher {
    fn drop(&mut self) {
        if let Err(err) = self.client.disconnect(None) {
            // We don't really care about errors here, but let's make rustc happy.
            error!("{:?}", err);
        }
    }
}
