use std::{env, process, path::Path, fs, time::Duration};

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

fn handle_data(data: &str) {
    println!("Received data: {}", data);
}

fn read_loop(device_path: &Path) {
    loop {
        match fs::read_to_string(device_path) {
            Ok(data) => handle_data(&data.trim()),
            Err(err) => {
                eprintln!("Cannot open device for reading: {}", err);
            },
        };

        std::thread::sleep(Duration::from_secs(1));
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let options = parse_args(args).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });

    println!("d0bby reading from {}", options.device_name);
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
