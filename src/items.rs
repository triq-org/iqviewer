// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- Item handling.

use std::fs;
//use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::usize;

use iced::widget::image::Handle;

use crate::dirs::read_dir_iq;
use crate::plot_ffi::Plot;
use crate::watcher;

/// Basically a Vec<FileItem> but maintains a filter and selection.
#[derive(Default)]
pub struct ItemList {
    items: Vec<FileItem>,
    prev_selection: usize,
    selection: usize,
    filter_map: Vec<usize>,
    filter_text: String,
    watcher: Option<watcher::FolderWatcher>,
    recent_folders: Vec<PathBuf>,
}

impl ItemList {
    pub fn len(&self) -> usize {
        self.filter_map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.filter_map.is_empty()
    }

    pub fn unfiltered_len(&self) -> usize {
        self.items.len()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        // also validates selection
        self.set_filter("");
        // unwatch all if we have a watcher, nothing to do otherwise
        self.recent_folders.drain(..);
        self.watcher.as_mut().map(|w| w.unwatch_all());
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for item in iter {
            self.push(item)
        }
    }

    fn refresh_all<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for item in iter {
            self.refresh(&item)
        }
    }

    fn remove_all<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for item in iter {
            self.remove(&item)
        }
    }

    pub fn push(&mut self, path: PathBuf) {
        if path.is_file() {
            self.items.push(FileItem::new(
                path.canonicalize().expect("Canonicalize path"),
            ));
        } else {
            match read_dir_iq(&path) {
                Ok(files) => {
                    for path in files {
                        self.items.push(FileItem::new(
                            path.canonicalize().expect("Canonicalize path"),
                        ));
                    }

                    // stash recent folders and try to apply
                    self.recent_folders.push(path);
                    if let Some(watcher) = self.watcher.as_mut() {
                        for path in self.recent_folders.drain(..) {
                            watcher.watch(path);
                        }
                    }
                }
                Err(err) => {
                    println!("Read error {err:?}");
                }
            }
        }
        self.apply_filter();
    }

    fn refresh(&mut self, path: &Path) {
        for item in self.items.iter_mut() {
            if item.path == path {
                item.refresh();
            }
        }
    }

    fn remove(&mut self, path: &Path) {
        self.items.retain(|item| {
            item.path != path // Note: path needs to be canonical
        });
        // also validates selection
        self.apply_filter();
    }

    pub fn get(&self, index: usize) -> Option<&FileItem> {
        self.filter_map.get(index).and_then(|&i| self.items.get(i))
    }

    pub fn filter(&self) -> &str {
        &self.filter_text
    }

    pub fn set_filter(&mut self, filter: &str) {
        self.filter_text = filter.to_ascii_lowercase();
        self.apply_filter();
    }

    pub fn count_watches(&self) -> usize {
        self.watcher.as_ref().map(|w| w.len()).unwrap_or_default()
    }

    pub fn count_marked(&self) -> usize {
        self.items
            .iter()
            .fold(0, |acc, t| if t.has_mark { acc + 1 } else { acc })
    }

    pub fn count_to_delete(&self) -> usize {
        self.items
            .iter()
            .fold(0, |acc, t| if t.has_delete { acc + 1 } else { acc })
    }

    pub fn has_selection(&self) -> bool {
        self.selection < self.len()
    }

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn set_selection(&mut self, index: usize) {
        self.selection = index.min(self.len().saturating_sub(1));
        self.prev_selection = self.selection;
    }

    pub fn inc_selection(&mut self, offset: usize) {
        self.set_selection(self.selection.saturating_add(offset));
    }

    pub fn dec_selection(&mut self, offset: usize) {
        self.set_selection(self.selection.saturating_sub(offset));
    }

    pub fn selected_remove(&mut self) {
        if self.has_selection() {
            let index = self.filter_map[self.selection];
            self.items.remove(index);
            // also validates selection
            self.apply_filter();
        }
    }

    pub fn selected_toggle_mark(&mut self) {
        self.selected_mut().map(FileItem::toggle_mark);
    }

    pub fn selected_toggle_delete(&mut self) {
        self.selected_mut().map(FileItem::toggle_delete);
    }

    pub fn selected(&self) -> Option<&FileItem> {
        self.filter_map.get(self.selection).and_then(|&i| self.items.get(i))
    }

    pub fn selected_mut(&mut self) -> Option<&mut FileItem> {
        self.filter_map.get(self.selection).and_then(|&i| self.items.get_mut(i))
    }

    pub fn iter(&self) -> impl Iterator<Item = &FileItem> {
        self.filter_map.iter().map(|&i| &self.items[i])
    }

    /// Rebuild filter_map, try to maintain the selection.
    fn apply_filter(&mut self) {
        let filter = self.filter_text.split_ascii_whitespace();
        // get previous selection index
        let prev_index = self.filter_map.get(self.prev_selection).copied().unwrap_or_default();
        // new selection
        let mut selection = 0;
        self.filter_map.clear();
        for (i, item) in self.items.iter().enumerate() {
            // test filter condition
            if item.matches_all(filter.clone()) {
                self.filter_map.push(i);
                // move selection along until we reach the previous index
                if i < prev_index {
                    selection += 1;
                }
            }
        }
        self.set_selection(selection);
    }

    pub fn move_marked_to(&mut self, dst: PathBuf) {
        self.items.retain(|item| {
            if item.has_mark {
                // NOTE: only works if the rename points to the same drive, otherwise needs fs::copy and fs::remove_file.
                if let Some(filename) = item.as_ref().file_name() {
                    let dst_file = dst.join(filename);
                    if let Err(err) = fs::rename(&item, &dst_file) {
                        println!("File move error: {:?}", err);
                        true // errored thus retain
                    } else {
                        false // remove
                    }
                } else {
                    true // errored thus retain
                }
            } else {
                true // retain
            }
        });
        self.apply_filter();
    }

    pub fn delete_marked(&mut self) {
        self.items.retain(|item| {
            if item.has_delete {
                if let Err(err) = fs::remove_file(&item) {
                    println!("File delete error: {:?}", err);
                    true // errored thus retain
                } else {
                    false // remove
                }
            } else {
                true // retain
            }
        });
        self.apply_filter();
    }

    pub fn watcher_event(&mut self, event: watcher::WatcherEvent) {
        match event {
            watcher::WatcherEvent::Ready(watcher) => {
                self.watcher = Some(watcher);

                // apply recent folders, likely from startup args
                if let Some(watcher) = self.watcher.as_mut() {
                    for path in self.recent_folders.drain(..) {
                        watcher.watch(path);
                    }
                }
            }

            watcher::WatcherEvent::Added(path) => {
                self.watcher.as_mut().map(|w| w.added(path));
            }

            watcher::WatcherEvent::Removed(path) => {
                self.watcher.as_mut().map(|w| w.removed(path));
            }

            watcher::WatcherEvent::Create(paths) => {
                self.extend(paths);
            }

            watcher::WatcherEvent::Modify(paths) => {
                self.refresh_all(paths);
            }

            watcher::WatcherEvent::Remove(paths) => {
                self.remove_all(paths);
            }
        }
    }
}

pub struct FileItem {
    path: PathBuf,
    size: Option<u64>,
    sample_format: &'static str,
    sample_count: u64,
    center_freq: f64,
    sample_rate: f64,
    handle: Handle,
    has_mark: bool,
    has_delete: bool,
    metadata: String,
}

impl AsRef<Path> for FileItem {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl FileItem {
    pub fn new(path: PathBuf) -> Self {
        let size = if let Ok(metadata) = fs::metadata(&path) {
            Some(metadata.len())
        } else {
            None
        };

        let (bitmap, file_info) = Plot::thumbnail(&path);
        let handle = Handle::from_rgba(bitmap.width as u32, bitmap.height as u32, bitmap.pixels);

        let metadata = format!("{} {} {:.0}M {:.0}k", path.to_string_lossy(), file_info.sample_format, file_info.center_freq / 1_000_000.0, file_info.sample_rate / 1_000.0).to_ascii_lowercase();

        Self {
            path,
            size,
            sample_format: file_info.sample_format,
            sample_count: file_info.sample_count,
            center_freq: file_info.center_freq,
            sample_rate: file_info.sample_rate,
            handle,
            has_mark: false,
            has_delete: false,
            metadata,
        }
    }

    pub fn refresh(&mut self) {
        self.size = if let Ok(metadata) = fs::metadata(&self.path) {
            Some(metadata.len())
        } else {
            None
        };

        let (bitmap, file_info) = Plot::thumbnail(&self.path);
        self.handle = Handle::from_rgba(bitmap.width as u32, bitmap.height as u32, bitmap.pixels);

        self.sample_format = file_info.sample_format;
        self.sample_count = file_info.sample_count;
        self.center_freq = file_info.center_freq;
        self.sample_rate = file_info.sample_rate;

        self.metadata = format!("{} {} {:.0}M {:.0}k", self.path.to_string_lossy(), file_info.sample_format, file_info.center_freq / 1_000_000.0, file_info.sample_rate / 1_000.0).to_ascii_lowercase();
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn size(&self) -> Option<u64> {
        self.size
    }

    pub fn sample_format(&self) -> &'static str {
        self.sample_format
    }

    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    pub fn center_freq(&self) -> f64 {
        self.center_freq
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn handle(&self) -> &Handle {
        &self.handle
    }

    pub fn has_mark(&self) -> bool {
        self.has_mark
    }

    pub fn has_delete(&self) -> bool {
        self.has_delete
    }

    pub fn toggle_mark(&mut self) {
        self.has_mark = !self.has_mark;
    }

    pub fn toggle_delete(&mut self) {
        self.has_delete = !self.has_delete;
    }

    pub fn filename(&self) -> std::borrow::Cow<'_, str> {
        self.path
            .file_name()
            .map(std::ffi::OsStr::to_string_lossy)
            .unwrap_or_default()
    }

    /// Tests if all filter conditions match
    fn matches_all<'a>(&self, filter: impl IntoIterator<Item = &'a str>) -> bool {
        filter.into_iter().all(|filter| self.metadata.contains(filter))
    }
}
