use rusb::{DeviceHandle, Direction, GlobalContext as Context, Recipient, RequestType, Result};
use std::{sync::OnceLock, time::Duration};

const BRIGHTNESS_MIN: f32 = 400.0;
const BRIGHTNESS_MAX: f32 = 60000.0;
const BRIGHTNESS_RANGE: f32 = BRIGHTNESS_MAX - BRIGHTNESS_MIN;

const HID_REPORT_TYPE_FEATURE: u16 = 0x0300;

const HID_GET_REPORT: u8 = 0x01;
const HID_SET_REPORT: u8 = 0x09;

const SD_BRIGHTNESS_INTERFACE: u8 = 0x7;
const SD_REPORT_ID: u16 = 0x01;
const SD_PRODUCT_ID: u16 = 0x1114;
const SD_VENDOR_ID: u16 = 0x05ac;

pub fn has_display() -> bool {
    let studio_display = get_studio_display();
    studio_display.is_ok()
}

pub fn set_brightness(percent: u8) -> Result<()> {
    let display = get_studio_display().as_ref().map_err(Clone::clone)?;
    let buffer = get_request_data(percent_to_nits(percent));

    display.write_control(
        make_request(Direction::Out),
        HID_SET_REPORT,
        HID_REPORT_TYPE_FEATURE | SD_REPORT_ID,
        SD_BRIGHTNESS_INTERFACE.into(),
        &buffer,
        Duration::from_secs(1),
    )?;

    Ok(())
}

pub fn get_brightness() -> Result<u8> {
    let display = get_studio_display().as_ref().map_err(Clone::clone)?;
    let mut buffer: [u8; 7] = [0; 7];

    display.read_control(
        make_request(Direction::In),
        HID_GET_REPORT,
        HID_REPORT_TYPE_FEATURE | SD_REPORT_ID,
        SD_BRIGHTNESS_INTERFACE.into(),
        &mut buffer,
        Duration::from_secs(1),
    )?;

    let nit_bytes: [u8; 2] = [buffer[1], buffer[2]];
    Ok(nits_to_percent(u16::from_le_bytes(nit_bytes)))
}

fn percent_to_nits(percent: u8) -> u16 {
    let factor = percent as f32 / 100.0;
    let scaled = BRIGHTNESS_RANGE * factor;
    (BRIGHTNESS_MIN + scaled) as u16
}

fn nits_to_percent(nits: u16) -> u8 {
    let scaled = nits as f32 - BRIGHTNESS_MIN;
    let factor = scaled / BRIGHTNESS_RANGE;
    (factor * 100.0) as u8
}

fn make_request(dir: Direction) -> u8 {
    rusb::request_type(dir, RequestType::Class, Recipient::Interface)
}

fn get_request_data(nits: u16) -> [u8; 7] {
    let mut bytes: [u8; 7] = [0; 7];
    let brightness = nits.to_le_bytes();
    bytes[0] = 0x01;
    bytes[1] = brightness[0];
    bytes[2] = brightness[1];
    return bytes;
}

fn get_studio_display() -> &'static Result<DeviceHandle<Context>> {
    static DISPLAY: OnceLock<Result<DeviceHandle<Context>>> = OnceLock::new();
    DISPLAY.get_or_init(|| {
        let mut dev = rusb::open_device_with_vid_pid(SD_VENDOR_ID, SD_PRODUCT_ID)
            .ok_or(rusb::Error::NoDevice)?;
        dev.set_active_configuration(0)?;
        dev.set_auto_detach_kernel_driver(true)?;
        dev.claim_interface(SD_BRIGHTNESS_INTERFACE)?;
        Ok(dev)
    })
}

#[cfg(test)]
mod tests {
    use super::{nits_to_percent, percent_to_nits, BRIGHTNESS_MAX, BRIGHTNESS_MIN};

    #[test]
    fn test_percent_to_nits() {
        assert_eq!(percent_to_nits(0), BRIGHTNESS_MIN as u16);
        assert_eq!(percent_to_nits(50), 30200);
        assert_eq!(percent_to_nits(100), BRIGHTNESS_MAX as u16);
    }

    #[test]
    fn test_nits_to_percent() {
        assert_eq!(nits_to_percent(BRIGHTNESS_MIN as u16), 0);
        assert_eq!(nits_to_percent(30200), 50);
        assert_eq!(nits_to_percent(BRIGHTNESS_MAX as u16), 100);
    }
}
