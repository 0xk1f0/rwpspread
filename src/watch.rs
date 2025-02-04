use crate::Wayland;
use crossbeam_channel::Sender;
use inotify::{Inotify, WatchMask};
use std::path::PathBuf;
use std::thread;
use std::thread::JoinHandle;

pub struct Watcher;
impl Watcher {
    pub fn monitors(mut wayland: Wayland, tx: Sender<bool>) -> Result<JoinHandle<()>, String> {
        let thread_handle = thread::Builder::new()
            .name("rwp_monitors".to_string())
            .spawn(move || match wayland.refresh() {
                Ok(resplit) => {
                    if resplit {
                        if let Err(err) = tx.send(true) {
                            eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                }
            })
            .map_err(|_| "thread: failed to start rwp_monitors")?;

        return Ok(thread_handle);
    }
    pub fn file(path: PathBuf, tx: Sender<bool>) -> Result<JoinHandle<()>, String> {
        let thread_handle = thread::Builder::new()
            .name("rwp_file".to_string())
            .spawn(move || match Watcher::source(&path) {
                Ok(resplit) => {
                    if resplit {
                        if let Err(err) = tx.send(true) {
                            eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{}: \x1B[91m{}\x1B[39m", "rwpspread", err);
                }
            })
            .map_err(|_| "thread: failed to start rwp_file")?;

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
