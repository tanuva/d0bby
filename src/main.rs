pub(crate) mod metervalues;

use std::{env, process, path::Path, fs, time::Duration};
use log::{debug, info, error};
use metervalues::MeterValues;

struct Options {
    device_name: String
}

fn parse_args(args: Vec<String>) -> Result<Options, &'static str> {
    if args.len() != 2 {
        return Err("Expected exactly 1 argument: the device to read from");
    }

    let options = Options {
        device_name: args[1].to_string()
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

fn handle_data(data: &str) {
    debug!("Received data: {}", data);
    let metered = parse_data(data);
}

fn read_loop(device_path: &Path) {
    loop {
        match fs::read_to_string(device_path) {
            Ok(data) => handle_data(&data.trim()),
            Err(err) => {
                error!("Cannot open device for reading: {}", err);
            },
        };

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let options = parse_args(args).unwrap_or_else(|err| {
        error!("{}", err);
        process::exit(1);
    });

    info!("Reading from {}", options.device_name);
    read_loop(&Path::new(&options.device_name));
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
}
