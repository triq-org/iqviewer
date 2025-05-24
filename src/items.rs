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

/// Basically a Vec<FileItem> but maintains a selection.
#[derive(Default)]
pub struct ItemList {
    items: Vec<FileItem>,
    selection: usize,
}

//impl Deref for ItemList {
//    type Target = Vec<FileItem>;
//
//    fn deref(&self) -> &Self::Target {
//        &self.items
//    }
//}
//
//impl DerefMut for ItemList {
//    fn deref_mut(&mut self) -> &mut Self::Target {
//        &mut self.items
//    }
//}

impl ItemList {
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.selection = 0;
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = PathBuf>,
    {
        for item in iter {
            self.push(item)
        }
    }

    pub fn push(&mut self, path: PathBuf) {
        if path.is_file() {
            self.items.push(FileItem::new(path));
        } else {
            let files = read_dir_iq(path).unwrap();
            for path in files {
                self.items.push(FileItem::new(path));
            }
        }
    }

    pub fn get(&self, index: usize) -> Option<&FileItem> {
        self.items.get(index)
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

    pub fn selection(&self) -> usize {
        self.selection
    }

    pub fn set_selection(&mut self, index: usize) {
        self.selection = index.min(self.len() - 1);
    }

    pub fn inc_selection(&mut self, offset: usize) {
        self.selection = self.selection.saturating_add(offset).min(self.len() - 1);
    }

    pub fn dec_selection(&mut self, offset: usize) {
        self.selection = self.selection.saturating_sub(offset);
    }

    pub fn selected(&self) -> Option<&FileItem> {
        self.items.get(self.selection)
    }

    pub fn selected_mut(&mut self) -> Option<&mut FileItem> {
        self.items.get_mut(self.selection)
    }

    pub fn iter(&self) -> impl Iterator<Item = &FileItem> {
        self.items.iter()
    }

    #[allow(unused)]
    pub fn filtered<'a>(&'a self, filter: &str) -> impl Iterator<Item = &'a FileItem> {
        self.items
            .iter()
            .filter(move |&t| t.filename().contains(filter))
    }

    pub fn move_marked_to(&mut self, dst: PathBuf) {
        self.items.retain(|item| {
            if item.has_mark {
                // NOTE: only works if the rename points to the same drive, otherwise needs fs::copy and fs::remove_file.
                let filename = item.as_ref().file_name();
                if filename.is_none() {
                    return true; // errored thus retain
                }
                let dst_file = dst.join(filename.unwrap());
                if let Err(err) = fs::rename(&item, &dst_file) {
                    println!("File move error: {:?}", err);
                    true // errored thus retain
                } else {
                    false // remove
                }
            } else {
                true // retain
            }
        });
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
    }
}

pub struct FileItem {
    path: PathBuf,
    handle: Handle,
    has_mark: bool,
    has_delete: bool,
}

impl AsRef<Path> for FileItem {
    fn as_ref(&self) -> &Path {
        self.path.as_path()
    }
}

impl FileItem {
    pub fn new(path: PathBuf) -> Self {
        let (pixels, width, height) = Plot::thumbnail(&path);
        let handle = Handle::from_rgba(width as u32, height as u32, pixels);

        Self {
            path,
            handle,
            has_mark: false,
            has_delete: false,
        }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
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
}
