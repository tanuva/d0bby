pub(crate) mod meterpublisher;
pub(crate) mod metervalues;

use log::{debug, error, info};
use metervalues::MeterValues;
use std::{env, fs, path::Path, process, time::Duration};

use crate::meterpublisher::MeterPublisher;

struct Options {
    device_name: String,
}

fn parse_args(args: Vec<String>) -> Result<Options, &'static str> {
    if args.len() != 2 {
        return Err("Expected exactly 1 argument: the device to read from");
    }

    let options = Options {
        device_name: args[1].to_string(),
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

    let url = "tcp://idefix.local:1883";
    let identifier = "hausanschluss";
    let mut publisher = MeterPublisher::new(url, identifier).unwrap_or_else(|err| {
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
        let device_name = "/dev/zero";
        let args = ["./d0bby".to_string(), device_name.to_string()].to_vec();
        let result = parse_args(args);
        assert!(result.is_ok());
        let options = result.unwrap();
        assert_eq!(options.device_name, device_name);
    }

    fn test_parse_data() {
        todo!()
    }
}
