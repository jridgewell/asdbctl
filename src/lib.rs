#![deny(clippy::all)]

mod shared;

#[macro_use]
extern crate napi_derive;

#[napi]
pub fn has_display() -> bool {
    shared::has_display()
}

#[napi]
pub fn set_brightness(percent: u8) -> napi::Result<()> {
    shared::set_brightness(percent).map_err(into_js_error)
}

#[napi]
pub fn get_brightness() -> napi::Result<u8> {
    shared::get_brightness().map_err(into_js_error)
}

fn into_js_error(err: rusb::Error) -> napi::Error {
    napi::Error::from_reason(err.to_string())
}

