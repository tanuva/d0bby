pub(crate) mod meterpublisher;
pub(crate) mod metervalues;

use json;
use log::{debug, error, info};
use metervalues::MeterValues;
use std::{env, fs, path::Path, process, time::Duration};

use crate::meterpublisher::MeterPublisher;

struct Options {
    device_name: String,
    broker_url: String,
    instance_id: String,
}

fn read_config(file_path: &Path) -> Result<Options, String> {
    let config_text = match fs::read_to_string(file_path) {
        Ok(text) => text,
        Err(err) => {
            return Err(format!("Cannot read configuration: {}", err));
        }
    };
    let config_json = match json::parse(&config_text) {
        Ok(json) => json,
        Err(err) => {
            return Err(format!("Cannot parse configuration: {}", err));
        }
    };

    for key in ["d0_device", "broker_url", "identifier"] {
        if !config_json.has_key(key) {
            return Err(format!("Cannot find key '{}' in {:?}", key, file_path));
        }
    }

    return Ok(Options {
        device_name: config_json["d0_device"].to_string(),
        broker_url: config_json["broker_url"].to_string(),
        instance_id: config_json["identifier"].to_string(),
    });
}

fn parse_args(args: Vec<String>) -> Result<Options, String> {
    if args.len() != 2 {
        return Err(format!("Usage: {} /path/to/d0_device", args[0]));
    }

    let options = match read_config(&Path::new(&args[1])) {
        Ok(options) => options,
        Err(err) => {
            return Err(err);
        }
    };

    return Ok(options);
}

fn parse_data(data: &str) -> Result<MeterValues, String> {
    // TODO Fake it till you make it!
    Ok(MeterValues {
        in_kwh: 1337.0,
        out_kwh: 42.0,
    })
}

fn read_loop(device_path: &Path, publisher: &mut MeterPublisher) {
    let data = match fs::read_to_string(device_path) {
        Ok(data) => data,
        Err(err) => {
            error!("Cannot open device for reading: {}", err);
            return;
        }
    };

    debug!("Received data: {}", data);

    let metered = match parse_data(&data) {
        Ok(metered) => metered,
        Err(err) => {
            error!("Cannot parse data: {}", err);
            return;
        }
    };

    if let Err(err) = publisher.publish(&metered) {
        error!("Cannot publish data: {}", err);
        return;
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let options = parse_args(args).unwrap_or_else(|err| {
        error!("{}", err);
        process::exit(1);
    });

    let mut publisher = MeterPublisher::new(&options.broker_url, &options.instance_id)
        .unwrap_or_else(|err| {
            error!("{}", err);
            process::exit(1);
        });

    publisher.publish_discovery().unwrap_or_else(|err| {
        error!("{}", err);
        process::exit(1);
    });

    info!("Reading from {}", options.device_name);

    loop {
        read_loop(&Path::new(&options.device_name), &mut publisher);
        std::thread::sleep(Duration::from_secs(10));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let config_file = "config.sample.json";
        let args = ["./d0bby".to_string(), config_file.to_string()].to_vec();
        let result = parse_args(args);
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.device_name, "/path/to/d0_device");
        assert_eq!(options.broker_url, "tcp://localhost:1883");
        assert_eq!(options.instance_id, "main_electricity_meter");
    }

    fn test_parse_data() {
        todo!()
    }
}
