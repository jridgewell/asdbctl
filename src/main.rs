mod shared;

use shared::{get_brightness, has_display, set_brightness};
use std::{env, error::Error};

pub fn main() -> Result<(), Box<dyn Error>> {
    if !has_display() {
        return Err("Did not detect Studio Display".into());
    }

    let mut args = env::args().skip(1);
    match args.next().as_deref() {
        Some("get") => {
            let brightness = get_brightness()?;
            println!("brightness is {brightness}");
        }
        Some("set") => {
            let brightness = args
                .next()
                .ok_or("missing required brightness arg".to_owned())
                .and_then(|i| i.parse::<u8>().map_err(|e| e.to_string()))?;
            set_brightness(brightness)?;
        }
        Some(arg) => {
            return Err(format!("unknown command {}", arg).into());
        }
        None => {
            eprintln!("asdbctl get              # get brigthness");
            eprintln!("asdbctl set |brightness| # set brigthness");
            return Err("invalid usage".into());
        }
    }
    Ok(())
}
