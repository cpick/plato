mod helpers;

use std::io;
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use serde::{Serialize, Deserialize};
use serde_json::json;
use reqwest::blocking::Client;
use anyhow::{Error, Context, format_err};
use self::helpers::load_toml;

const SETTINGS_PATH: &str = "Settings.toml";

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
struct Settings {
    update_path: PathBuf,
    url: String,
}

fn main() -> Result<(), Error> {
    let mut args = env::args().skip(1);
    let _library_path = PathBuf::from(args.next()
                                         .ok_or_else(|| format_err!("missing argument: library path"))?);
    let _save_path = PathBuf::from(args.next()
                                      .ok_or_else(|| format_err!("missing argument: save path"))?);
    let wifi = args.next()
                   .ok_or_else(|| format_err!("missing argument: wifi status"))
                   .and_then(|v| v.parse::<bool>().map_err(Into::into))?;
    let online = args.next()
                     .ok_or_else(|| format_err!("missing argument: online status"))
                     .and_then(|v| v.parse::<bool>().map_err(Into::into))?;
    let settings = load_toml::<Settings, _>(SETTINGS_PATH)
                             .with_context(|| format!("can't load settings from {}", SETTINGS_PATH))?;

    if !online {
        if !wifi {
            let event = json!({
                "type": "notify",
                "message": "Establishing a network connection.",
            });
            println!("{}", event);
            let event = json!({
                "type": "setWifi",
                "enable": true,
            });
            println!("{}", event);
        } else {
            let event = json!({
                "type": "notify",
                "message": "Waiting for the network to come up.",
            });
            println!("{}", event);
        }
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
    }

    let client = Client::new();

    let sigterm = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&sigterm))?;

    loop {
        if sigterm.load(Ordering::Relaxed) {
            break;
        }

        let mut file_name = settings.update_path.file_name().unwrap(/* TODO */).to_owned();
        file_name.push(".update");
        let temporary_path = settings.update_path.with_file_name(file_name);

        {
            let mut file = File::create(&temporary_path)?; // NB remove on any early exit from here on
            let response = client.get(&settings.url)
                                 .send()
                                 .and_then(|body|
                                     body.error_for_status()?
                                         .copy_to(&mut file)
                                 );

            if let Err(err) = response {
                eprintln!("Can't download {}: {:#}.", settings.url, err);
                fs::remove_file(temporary_path).ok();
                continue;
            }
        }

        fs::rename(temporary_path, settings.update_path)?;
        break;
    }

    if !wifi {
        let event = json!({
            "type": "setWifi",
            "enable": false,
        });
        println!("{}", event);
    }

    Ok(())
}
