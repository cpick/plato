#[macro_use] mod geom;
mod unit;
mod color;
mod device;
mod framebuffer;
mod frontlight;
mod lightsensor;
mod battery;
mod input;
mod gesture;
mod helpers;
mod dictionary;
mod document;
mod library;
mod metadata;
mod symbolic_path;
mod rtc;
mod settings;
mod view;
mod font;
mod app;

use anyhow::Error;
use crate::app::run;
use std::os::unix::ffi::OsStringExt;

fn main() -> Result<(), Error> {
    if run()? {
        let args = std::env::args_os()
            .map(|arg| std::ffi::CString::new(arg.into_vec()).unwrap(/* TODO */))
            .collect::<Vec<_>>();
        nix::unistd::execvp(args.first().unwrap(/* TODO */), &args).unwrap(/* TODO */);
    }
    Ok(())
}
