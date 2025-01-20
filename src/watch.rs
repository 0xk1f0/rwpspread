use inotify::{Inotify, WatchMask};
use std::path::PathBuf;

pub struct Watcher;
impl Watcher {
    pub fn source(path: &PathBuf) -> Result<bool, String> {
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
