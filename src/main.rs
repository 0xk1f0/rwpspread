mod cli;
mod integrations;
mod wayland;
mod worker;

use cli::Config;
use inotify::{Inotify, WatchMask};
use integrations::helpers;
use std::panic;
use std::process;
use std::sync::mpsc::sync_channel;
use std::thread;
use wayland::MonitorConfig;
use worker::Worker;

fn run() -> Result<String, String> {
    // create new config
    let worker_config = Config::new()?;

    // connect to wayland
    let mut mon_conn = MonitorConfig::new()?;

    // get monitor info
    let mon_config = mon_conn.run()?;

    // check for backends if applicable
    if let Some(config) = worker_config {
        // check for backends if applicable
        if let Some(run_config) = &config.backend {
            if !helpers::is_installed(&run_config.to_string()) {
                return Err(format!("{} is not installed", &run_config.to_string()));
            }
        }

        // create and execute worker
        let mut worker = Worker::new();
        worker.run(&config, mon_config)?;

        // check for watchdog bool
        if config.daemon {
            // interrupt arc for source watch thread
            let (tx, rx) = sync_channel(2);

            // check if we watch source
            if config.watch {
                // make new thread loop for source watch
                let tx = tx.clone();
                thread::Builder::new()
                    .name("source_watch".to_string())
                    .spawn(move || loop {
                        let mut buffer = [0; 1024];
                        let mut inotify = Inotify::init().expect("inotify: failed to initialize");
                        inotify
                            .watches()
                            .add(
                                &config.raw_input_path,
                                WatchMask::MODIFY
                                    | WatchMask::DELETE
                                    | WatchMask::CREATE
                                    | WatchMask::MOVE
                                    | WatchMask::MOVE_SELF
                                    | WatchMask::DELETE_SELF
                                    | WatchMask::DONT_FOLLOW,
                            )
                            .expect("inotify: failed to add watch");

                        let events = inotify
                            .read_events_blocking(&mut buffer)
                            .expect("inotify: failed to read events");

                        if events.count() > 0 {
                            inotify.close().expect("inotify: failed to close");
                            tx.send("resplit")
                                .expect("source_watch: failed to notify resplit");
                        }
                    })
                    .map_err(|_| "failed to start output_watch thread")?;
            }

            thread::Builder::new()
                .name("output_watch".to_string())
                .spawn(move || {
                    let tx = tx.clone();
                    loop {
                        if mon_conn.refresh().expect("wayland: refresh error") {
                            tx.send("resplit")
                                .expect("output_watch: failed to notify resplit");
                        };
                    }
                })
                .map_err(|_| "failed to start output_watch thread")?;

            loop {
                // rerun if config changed or screens changed
                if let Ok(_) = rx.recv() {
                    // redetect screens
                    let mons = MonitorConfig::new()?.run()?;
                    // rerun splitter
                    if let Some(conf) = Config::new()? {
                        worker.run(&conf, mons)?;
                    }
                } else {
                    return Err("watcher threads panicked".to_string());
                }
            }
        }
    } else {
        // since no runtime config was found, return info
        let mut result = String::from("Found the following displays:\n");
        for monitor in mon_config {
            result.push_str(&format!("- {}\n", monitor));
        }
        result.push_str("Supported versions:\n");
        result.push_str(&format!(
            "- \x1B[32mhyprpaper\x1B[39m: {}\n- \x1B[32mwpaperd\x1B[39m: {}\n- \x1B[32mswaybg\x1B[39m: {}",
            env!("HYPRPAPER_VERSION"),
            env!("WPAPERD_VERSION"),
            env!("SWAYBG_VERSION")
        ));
        return Ok(result);
    }

    Ok("".to_string())
}

fn main() {
    panic::set_hook(Box::new(|panic| {
        if let Some(s) = panic.payload().downcast_ref::<&str>() {
            eprintln!("{}: panic: \x1B[91m{s:?}\x1B[39m", "rwpspread");
        } else if let Some(s) = panic.payload().downcast_ref::<String>() {
            eprintln!("{}: panic: \x1B[91m{s:?}\x1B[39m", "rwpspread");
        } else {
            eprintln!("{}: panic: \x1B[91m{}\x1B[39m", "rwpspread", "panicked");
        }
        process::exit(1);
    }));

    match run() {
        Ok(ok) => {
            println!("{}", ok);
            process::exit(0);
        }
        Err(err) => {
            eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
            process::exit(1);
        }
    }
}
