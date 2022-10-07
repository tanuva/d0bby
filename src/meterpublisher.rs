use crate::metervalues::MeterValues;
use json::{self, object};
use log::error;
use paho_mqtt as mqtt;
use std::time::Duration;

pub struct MeterPublisher {
    client: mqtt::Client,
    identifier: String,
    topics: Topics,
}

struct Topics {
    state: String,
    discovery_in: String,
    discovery_out: String,
}

impl MeterPublisher {
    pub fn new(url: &str, identifier: &str) -> Result<MeterPublisher, String> {
        let client = match mqtt::Client::new(url) {
            Ok(client) => client,
            Err(err) => {
                return Err(format!("{:?}", err));
            }
        };

        let conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_secs(20))
            .clean_session(true)
            // TODO Will message
            .finalize();

        if let Err(err) = client.connect(conn_opts) {
            return Err(format!("Cannot connect to {}: {:?}", url, err));
        }

        Ok(MeterPublisher {
            client,
            identifier: identifier.to_string(),
            topics: Topics {
                state: format!("d0bby/{}/state", identifier),
                discovery_in: format!("homeassistant/sensor/d0bby/{}_in/config", identifier),
                discovery_out: format!("homeassistant/sensor/d0bby/{}_out/config", identifier),
            },
        })
    }

    pub fn publish(&self, values: &MeterValues) -> Result<(), String> {
        if !self.client.is_connected() {
            if let Err(err) = self.client.reconnect() {
                return Err(format!("{:?}", err));
            }
        }

        let payload = object! {
            state: "ON",
            in_kwh: values.in_kwh,
            out_kwh: values.out_kwh,
        };

        let msg = mqtt::Message::new(&self.topics.state, json::stringify(payload), 0);
        if let Err(err) = self.client.publish(msg) {
            return Err(format!("{:?}", err));
        }

        Ok(())
    }

    pub fn publish_discovery(&self) -> Result<(), String> {
        if !self.client.is_connected() {
            if let Err(err) = self.client.reconnect() {
                return Err(format!("{:?}", err));
            }
        }

        let instance_identifier = format!("d0bby_{}", self.identifier);
        let payload = object! {
            schema: "json",

            availability_topic: self.topics.state.to_string(),
            availability_template: "{{ value_json.state }}",
            payload_available: "ON",
            payload_not_available: "OFF",

            state_topic: self.topics.state.to_string(),
            unit_of_measurement: "kWh",
            device_class: "energy",
            device: {
                identifiers: instance_identifier.to_string(),
                manufacturer: "Marcel Kummer",
                model: "d0bby",
                name: "d0bby",
            }
        };

        let mut payload_in = payload.clone();
        if let Err(err) = payload_in.insert("unique_id", format!("{}_in", instance_identifier)) {
            return Err(format!("{:?}", err));
        }
        if let Err(err) = payload_in.insert("name", "Input") {
            return Err(format!("{:?}", err));
        }
        if let Err(err) = payload_in.insert("value_template", "{{ value_json.in_kwh }}") {
            return Err(format!("{:?}", err));
        }

        let msg = mqtt::Message::new_retained(&self.topics.discovery_in, payload_in.dump(), 0);
        if let Err(err) = self.client.publish(msg) {
            return Err(format!("{:?}", err));
        }

        let mut payload_out = payload.clone();
        if let Err(err) = payload_out.insert("unique_id", format!("{}_out", instance_identifier)) {
            return Err(format!("{:?}", err));
        }
        if let Err(err) = payload_out.insert("name", "Output") {
            return Err(format!("{:?}", err));
        }
        if let Err(err) = payload_out.insert("value_template", "{{ value_json.out_kwh }}") {
            return Err(format!("{:?}", err));
        }

        let msg = mqtt::Message::new_retained(&self.topics.discovery_out, payload_out.dump(), 0);
        if let Err(err) = self.client.publish(msg) {
            return Err(format!("{:?}", err));
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
