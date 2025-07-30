// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- Folder watcher.

use notify::Watcher;
use std::ops::Deref;
use std::path::PathBuf;

use iced::futures::channel::mpsc;
use iced::futures::sink::SinkExt;
use iced::futures::{Stream, StreamExt};
use iced::stream;

#[derive(Debug, Clone)]
pub enum WatcherEvent {
    Ready(FolderWatcher),
    Added(PathBuf),
    Removed(PathBuf),
    Create(Vec<PathBuf>),
    Modify(Vec<PathBuf>),
    Remove(Vec<PathBuf>),
}

#[derive(Debug, Clone)]
enum Cmd {
    Watch(PathBuf),
    Unwatch(PathBuf),
}

pub fn watcher_subscription() -> impl Stream<Item = WatcherEvent> {
    // Note: this queues and delays if n > 0, why?
    stream::channel(0, async |mut output| {
        // Create a channel for watch commands
        let (sender, mut receiver) = mpsc::channel(100);
        // Send the sender back to the application
        output
            .send(WatcherEvent::Ready(FolderWatcher::new(sender)))
            .await
            .expect("Send Ready event");

        let (mut tx, mut rx) = mpsc::channel::<notify::Result<notify::Event>>(100);

        // automatically select the best implementation
        let mut watcher = notify::recommended_watcher(move |event| {
            tx.try_send(event).expect("Send watcher event");
        })
        .expect("Create watcher");

        loop {
            iced::futures::select! {
                res = receiver.select_next_some() => {
                    match res {
                        Cmd::Watch(path) => {
                            if watcher.watch(&path, notify::RecursiveMode::NonRecursive).is_ok() {
                                output.send(WatcherEvent::Added(path)).await.expect("Send Added event");
                            }
                        }
                        Cmd::Unwatch(path) => {
                            if watcher.unwatch(&path).is_ok() {
                                output.send(WatcherEvent::Removed(path)).await.expect("Send Removed event");
                            }
                        }
                    }
                }
                res = rx.select_next_some() => {
                    match res {
                        Ok(event) => {
                            match event {
                                notify::Event { kind: notify::EventKind::Create(notify::event::CreateKind::File), paths, ..} => {
                                    output.send(WatcherEvent::Create(paths)).await.expect("Send Create event");
                                }
                                notify::Event { kind: notify::EventKind::Modify(notify::event::ModifyKind::Data(_)), paths, ..} => {
                                    output.send(WatcherEvent::Modify(paths)).await.expect("Send Modify event");
                                }
                                notify::Event { kind: notify::EventKind::Remove(notify::event::RemoveKind::File), paths, ..} => {
                                    output.send(WatcherEvent::Remove(paths)).await.expect("Send Remove event");
                                }
                                notify::Event { .. } => {}
                            }
                        }
                        Err(e) => {
                            println!("watch error: {:?}", e);
                        }
                    }

                }
            };
        }
    })
}

#[derive(Debug, Clone)]
pub struct FolderWatcher {
    sender: mpsc::Sender<Cmd>,
    paths: Vec<PathBuf>,
}

impl Deref for FolderWatcher {
    type Target = Vec<PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.paths
    }
}

impl FolderWatcher {
    fn new(sender: mpsc::Sender<Cmd>) -> Self {
        Self {
            sender,
            paths: vec![],
        }
    }

    pub fn added(&mut self, path: PathBuf) {
        self.paths.push(path);
    }

    pub fn removed(&mut self, path: PathBuf) {
        self.paths.retain(|item| item != &path)
    }

    pub fn watch(&mut self, path: PathBuf) {
        self.sender
            .try_send(Cmd::Watch(path))
            .expect("Send Watch command")
    }

    #[allow(unused)]
    pub fn unwatch(&mut self, path: PathBuf) {
        self.sender
            .try_send(Cmd::Unwatch(path))
            .expect("Send Unwatch command")
    }

    pub fn unwatch_all(&mut self) {
        for path in self.paths.drain(..) {
            self.sender
                .try_send(Cmd::Unwatch(path))
                .expect("Send Unwatch command")
        }
    }
}
