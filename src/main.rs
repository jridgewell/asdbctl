use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use std::time::Duration;
use rusb::{GlobalContext as Context, Device};

const MIN_BRIGHTNESS: u16 = 400;
const MAX_BRIGHTNESS: u16 = 60000;

const _HID_REPORT_TYPE_INPUT: u16 = 0x0100;
const _HID_REPORT_TYPE_OUTPUT: u16 = 0x0200;
const HID_REPORT_TYPE_FEATURE: u16 = 0x0300;

const HID_GET_REPORT: u8 = 0x01;
const HID_SET_REPORT: u8 = 0x09;

const SD_BRIGHTNESS_INTERFACE: u8 = 0x7;
const SD_PRODUCT_ID: u16 = 0x1114;
const SD_VENDOR_ID: u16 = 0x05ac;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Display verbose information
    verbose: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    /// Get and display the current brightness level
    Get,
    /// Set the brightness level
    Set(SetCommandArgs),
}

fn percent(input: &str) -> Result<u8, String> {
    clap_num::number_range(input, 0, 100)
}

#[derive(Args)]
struct SetCommandArgs {
    /// Brightness level in percent 0-100
    #[clap(value_parser=percent)]
    brightness: u8,
}

fn percent_to_nits(brightness_percent: u8) -> u16 {
    let factor = brightness_percent as f32 / 100.0;
    let brightness_range = MAX_BRIGHTNESS - MIN_BRIGHTNESS;
    let brightness_value = MIN_BRIGHTNESS + (brightness_range as f32 * factor) as u16;
    return brightness_value;
}

fn get_request_data(nits: u16) -> [u8; 7] {
    let mut result: [u8; 7] = [0; 7];
    let le_bytes = nits.to_le_bytes();
    result[0] = 0x01;
    result[1] = le_bytes[0];
    result[2] = le_bytes[1];
    //let result = [ 0x01, 0x90, 0x01, 0x00, 0x00, 0x00, 0x00 ]; // min value
    //let result = [ 0x01, 0x60, 0xea, 0x00, 0x00, 0x00, 0x00 ]; // max value
    return result;
}

fn get_studio_display() -> Result<Device<Context>> {
    let usb_devices = rusb::devices().unwrap();
    for device in usb_devices.iter() {
        let device_desc = device.device_descriptor().unwrap();
        if device_desc.product_id() == SD_PRODUCT_ID && device_desc.vendor_id() == SD_VENDOR_ID {
            return Ok(device);
        }
    }
    Err(anyhow!("No Apple Studio Display connected"))
}

fn set_brightness(dev: &Device<Context>, nits: u16) -> Result<()> {
    let buffer = get_request_data(nits);
    let mut handle = dev.open().unwrap();
    // handle.detach_kernel_driver(SD_BRIGHTNESS_INTERFACE).unwrap();
    handle.claim_interface(SD_BRIGHTNESS_INTERFACE).unwrap();
    let request_type = rusb::request_type(
        rusb::Direction::Out,
        rusb::RequestType::Class,
        rusb::Recipient::Interface,
    );
    handle.write_control(
        request_type,                   // bmRequestType
        HID_SET_REPORT,                 // bRequest
        HID_REPORT_TYPE_FEATURE | 0x01, // wValue        HID - Report Type and Report ID
        SD_BRIGHTNESS_INTERFACE.into(), // wIndex        HID - Interface
        &buffer,
        Duration::from_secs(1),
    ).unwrap();
    handle.release_interface(SD_BRIGHTNESS_INTERFACE).unwrap();
    // handle.attach_kernel_driver(SD_BRIGHTNESS_INTERFACE).unwrap();
    Ok(())
}

fn get_brightness(dev: &Device<Context>) -> Result<u16> {
    let mut buffer: [u8; 7] = [0; 7];
    let mut handle = dev.open().unwrap();
    // handle.detach_kernel_driver(SD_BRIGHTNESS_INTERFACE).unwrap();
    handle.claim_interface(SD_BRIGHTNESS_INTERFACE).unwrap();
    let request_type = rusb::request_type(
        rusb::Direction::In,
        rusb::RequestType::Class,
        rusb::Recipient::Interface,
    );
    handle.read_control(
        request_type,                   // bmRequestType
        HID_GET_REPORT,                 // bRequest
        HID_REPORT_TYPE_FEATURE | 0x01, // wValue        HID - Report Type and Report ID
        SD_BRIGHTNESS_INTERFACE.into(), // wIndex        HID - Interface
        &mut buffer,
        Duration::from_secs(1),
    ).unwrap();
    handle.release_interface(SD_BRIGHTNESS_INTERFACE).unwrap();
    // handle.attach_kernel_driver(SD_BRIGHTNESS_INTERFACE).unwrap();
    let nit_bytes: [u8; 2] = [buffer[1], buffer[2]];
    Ok(u16::from_le_bytes(nit_bytes))
}

fn main() -> Result<()> {
    let cli_args = Cli::parse();
    let studio_display = get_studio_display().unwrap();

    match &cli_args.command {
        Commands::Set(set_args) => {
            let nits = percent_to_nits(set_args.brightness);
            if cli_args.verbose.unwrap_or_default() {
                println!("Setting brightness to: {}", nits);
            }
            set_brightness(&studio_display, nits).unwrap();
        }
        Commands::Get => {
            let nits = get_brightness(&studio_display).unwrap();
            println!("Brightness is set to: {}", nits);
        }
    }

    return Ok(());
}
