// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- file and directory helper.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[rustfmt::skip]
pub const FORMATS: &[&str] = &[
    "cu4",
    "cs4",
    "cu8", "data", "complex16u",
    "cs8", "complex16s",
    "cu12",
    "cs12",
    "cu16",
    "cs16",
    "cu32",
    "cs32",
    "cu64",
    "cs64",
    "cf32", "cfile", "complex",
    "cf64",
    "sigmf",
];

#[rustfmt::skip]
pub fn is_iq_file(path: impl AsRef<Path>) -> bool {
    // TODO: should use triq::SampleFormat::from_path(path)
    path.as_ref().extension().is_some_and(|ext| {
        ext == "cu4"
            || ext == "cs4"
            || ext == "cu8" || ext == "data" || ext == "complex16u"
            || ext == "cs8" || ext == "complex16s"
            || ext == "cu12"
            || ext == "cs12"
            || ext == "cu16"
            || ext == "cs16"
            || ext == "cu32"
            || ext == "cs32"
            || ext == "cu64"
            || ext == "cs64"
            || ext == "cf32" || ext == "cfile" || ext == "complex"
            || ext == "cf64"
            || ext == "sigmf"
    })
}

pub fn read_dir_iq(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    let mut entries = entries
        .into_iter()
        .filter(|p| is_iq_file(p))
        .collect::<Vec<_>>();

    // The order in which `read_dir` returns entries is not guaranteed.
    // Sort entries by their path.
    entries.sort();

    Ok(entries)
}
