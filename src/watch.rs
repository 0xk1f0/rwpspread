use crate::Wayland;
use inotify::{Inotify, WatchMask};
use std::path::PathBuf;
use std::sync::mpsc::SyncSender;
use std::thread;
use std::thread::JoinHandle;

pub struct Watcher;
impl Watcher {
    pub fn output_watch(
        mut wayland: Wayland,
        tx: SyncSender<&'static str>,
    ) -> Result<JoinHandle<()>, String> {
        let thread_handle = thread::Builder::new()
            .name("output_watch".to_string())
            .spawn(move || loop {
                match wayland.refresh() {
                    Ok(resplit) => {
                        if resplit {
                            if let Err(err) = tx.send("resplit") {
                                eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                        break;
                    }
                }
            })
            .map_err(|_| "failed to start output_watch thread")?;

        return Ok(thread_handle);
    }
    pub fn source_watch(
        path: PathBuf,
        tx: SyncSender<&'static str>,
    ) -> Result<JoinHandle<()>, String> {
        let thread_handle = thread::Builder::new()
            .name("source_watch".to_string())
            .spawn(move || loop {
                match Watcher::source(&path) {
                    Ok(resplit) => {
                        if resplit {
                            if let Err(err) = tx.send("resplit") {
                                eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                                break;
                            }
                        }
                    }
                    Err(err) => {
                        eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                        break;
                    }
                }
            })
            .map_err(|_| "failed to start output_watch thread")?;

        return Ok(thread_handle);
    }
    fn source(path: &PathBuf) -> Result<bool, String> {
        let mut buffer = [0; 1024];
        let mut inotify = Inotify::init().map_err(|_| "inotify: failed to initialize")?;
        inotify
            .watches()
            .add(
                path,
                WatchMask::MODIFY
                    | WatchMask::DELETE
                    | WatchMask::CREATE
                    | WatchMask::MOVE
                    | WatchMask::MOVE_SELF
                    | WatchMask::DELETE_SELF
                    | WatchMask::DONT_FOLLOW,
            )
            .map_err(|_| "inotify: failed to add watch")?;

        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("inotify: failed to read events");
        inotify.close().map_err(|_| "inotify: failed to close")?;

        if events.count() > 0 {
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
