// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! triq -- TRansform I/Q data.
//!
//! The foreign function interface (C-API) which exposes this library.

use std::ffi::{CString, c_char, c_void};

#[allow(non_camel_case_types)]
type splt_t = c_void;

#[link(name = "triq")]
#[allow(unused)]
unsafe extern "C" {
    /// Construct a new `Spectrogram` plot which will load the provided file path and fill out
    /// all other fields with their defaults.
    ///
    /// # Note
    ///
    /// If the string passed in isn't a valid file path this will return a null pointer.
    ///
    /// # Safety
    ///
    /// Make sure you destroy the spectrogram with [`splt_destroy()`] once you are
    /// done with it.
    ///
    /// [`splt_destroy()`]: fn.splt_destroy.html
    fn splt_create(path: *const c_char) -> *mut splt_t;

    /// Get the sample format on the Spectrogram plot.
    fn splt_get_sample_format(plot: *const splt_t) -> u8;
    /// Get the sample count on the Spectrogram plot.
    fn splt_get_sample_count(plot: *const splt_t) -> u64;
    /// Get the center frequency on the Spectrogram plot.
    fn splt_get_center_freq(plot: *const splt_t) -> f64;
    /// Get the sampe rate on the Spectrogram plot.
    fn splt_get_sample_rate(plot: *const splt_t) -> f64;

    /// Get the width on the Spectrogram plot.
    fn splt_get_layout_width(plot: *const splt_t) -> u32;
    /// Get the height on the Spectrogram plot.
    fn splt_get_layout_height(plot: *const splt_t) -> u32;

    /// Get the dark_theme on the Spectrogram plot.
    fn splt_get_dark_theme(plot: *const splt_t) -> bool;
    /// Set the dark_theme on the Spectrogram plot.
    fn splt_set_dark_theme(plot: *mut splt_t, dark: bool);
    /// Get the origin on the Spectrogram plot.
    fn splt_get_origin(plot: *const splt_t) -> u32;
    /// Set the origin on the Spectrogram plot.
    fn splt_set_origin(plot: *mut splt_t, origin: u32);
    /// Get the zoom on the Spectrogram plot.
    fn splt_get_zoom(plot: *const splt_t) -> u32;
    /// Set the zoom on the Spectrogram plot.
    fn splt_set_zoom(plot: *mut splt_t, zoom: u32);
    /// Get the db_gain on the Spectrogram plot.
    fn splt_get_db_gain(plot: *const splt_t) -> f32;
    /// Set the db_gain on the Spectrogram plot.
    fn splt_set_db_gain(plot: *mut splt_t, db_gain: f32);
    /// Get the db_range on the Spectrogram plot.
    fn splt_get_db_range(plot: *const splt_t) -> f32;
    /// Set the db_range on the Spectrogram plot.
    fn splt_set_db_range(plot: *mut splt_t, db_range: f32);
    /// Get the cmap on the Spectrogram plot.
    fn splt_get_cmap(plot: *const splt_t) -> u32;
    /// Set the cmap on the Spectrogram plot.
    fn splt_set_cmap(plot: *mut splt_t, cmap: u32);
    /// Get the fft_size on the Spectrogram plot.
    fn splt_get_fft_size(plot: *const splt_t) -> u32;
    /// Set the fft_size on the Spectrogram plot.
    fn splt_set_fft_size(plot: *mut splt_t, fft_size: u32);
    /// Get the fft_window on the Spectrogram plot.
    fn splt_get_fft_window(plot: *const splt_t) -> u8;
    /// Set the fft_window on the Spectrogram plot.
    fn splt_set_fft_window(plot: *mut splt_t, fft_window_name: u8);
    /// Set the layout_size on the Spectrogram plot.
    fn splt_set_layout_size(plot: *mut splt_t, width: u32, height: u32);
    /// Get the direction on the Spectrogram plot.
    fn splt_get_layout_direction(plot: *const splt_t) -> u8;
    /// Set the direction on the Spectrogram plot.
    fn splt_set_layout_direction(plot: *mut splt_t, direction: u8);
    /// Get the plot_across on the Spectrogram plot.
    fn splt_get_layout_plot_across(plot: *const splt_t) -> u32;
    /// Set the plot_across on the Spectrogram plot.
    fn splt_set_layout_plot_across(plot: *mut splt_t, plot_across: u32);
    /// Get the histo_width on the Spectrogram plot.
    fn splt_get_layout_histo_width(plot: *const splt_t) -> u32;
    /// Set the histo_width on the Spectrogram plot.
    fn splt_set_layout_histo_width(plot: *mut splt_t, histo_width: u32);
    /// Get the deci_height on the Spectrogram plot.
    fn splt_get_layout_deci_height(plot: *const splt_t) -> u32;
    /// Set the deci_height on the Spectrogram plot.
    fn splt_set_layout_deci_height(plot: *mut splt_t, deci_height: u32);
    /// Get the ask_height on the Spectrogram plot.
    fn splt_get_layout_ask_height(plot: *const splt_t) -> u32;
    /// Set the ask_height on the Spectrogram plot.
    fn splt_set_layout_ask_height(plot: *mut splt_t, ask_height: u32);

    /// Draw a `Spectrogram` into a pixel buffer.
    fn splt_draw(plot: *mut splt_t, pixels: *mut u32, width: u32, height: u32);

    /// Destroy a `Spectrogram` once you are done with it.
    fn splt_destroy(plot: *mut splt_t);
}

use std::path::{Path, PathBuf};

#[rustfmt::skip]
const SAMPLE_FORMAT: &[&str] = &[
    "CU4",
    "CS4",
    "CU8",
    "CS8",
    "CU12",
    "CS12",
    "CU16",
    "CS16",
    "CU32",
    "CS32",
    "CU64",
    "CS64",
    "CF32",
    "CF64",
];

pub struct Plot {
    path: PathBuf,
    plot: *mut splt_t,
}

impl Drop for Plot {
    fn drop(&mut self) {
        unsafe {
            splt_destroy(self.plot);
        }
    }
}

#[allow(unused)]
impl Plot {
    pub fn with_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        let plot = Self::create_plot(path);

        Self {
            path: path.to_path_buf(),
            plot: plot,
        }
    }

    pub fn open(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        let plot = Self::create_plot(path);
        self.path = path.to_path_buf();
        self.plot = plot
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn thumbnail(path: impl AsRef<Path>) -> (Vec<u8>, usize, usize) {
        let path = path.as_ref();
        let plot = Self::create_plot(path);

        let width = 256;
        let height = 256;

        // Setup Spectroplot
        unsafe {
            // splt_set_dark_theme(plot, true);
            splt_set_layout_size(plot, width, height);
        }

        let width = unsafe { splt_get_layout_width(plot) } as usize;
        let height = unsafe { splt_get_layout_height(plot) } as usize;

        let mut pixels = vec![0; width * height];

        // Run Spectroplot
        unsafe {
            splt_draw(plot, pixels.as_mut_ptr(), width as u32, height as u32);
        }

        //println!("Rendered size: {} x {}", width, height);

        Self::pixels_toraw(pixels, width, height)
    }

    fn create_plot(path: impl AsRef<Path>) -> *mut splt_t {
        // FIXME: Check if this is a file or a folder...

        let path_str_c = CString::new(path.as_ref().as_os_str().as_encoded_bytes()).unwrap();

        let plot = unsafe { splt_create(path_str_c.as_ptr()) };

        // Setup Spectroplot
        unsafe {
            splt_set_dark_theme(plot, true);
        }

        plot
    }

    pub fn zoom(&self) -> usize {
        unsafe { splt_get_zoom(self.plot) as usize }
    }
    pub fn sample_format(&self) -> u8 {
        unsafe { splt_get_sample_format(self.plot) }
    }
    pub fn sample_count(&self) -> u64 {
        unsafe { splt_get_sample_count(self.plot) }
    }
    pub fn center_freq(&self) -> f64 {
        unsafe { splt_get_center_freq(self.plot) }
    }
    pub fn sample_rate(&self) -> f64 {
        unsafe { splt_get_sample_rate(self.plot) }
    }
    pub fn db_gain(&self) -> f32 {
        unsafe { splt_get_db_gain(self.plot) }
    }
    pub fn db_range(&self) -> f32 {
        unsafe { splt_get_db_range(self.plot) }
    }
    pub fn fft_size(&self) -> u32 {
        unsafe { splt_get_fft_size(self.plot) }
    }

    pub fn set_zoom(&self, zoom: u32) {
        unsafe { splt_set_zoom(self.plot, zoom) }
    }
    pub fn set_db_gain(&self, db_gain: f32) {
        unsafe { splt_set_db_gain(self.plot, db_gain) }
    }
    pub fn set_db_range(&self, db_range: f32) {
        unsafe { splt_set_db_range(self.plot, db_range) }
    }
    pub fn set_cmap(&self, cmap: u32) {
        unsafe { splt_set_cmap(self.plot, cmap) }
    }
    pub fn set_fft_size(&self, fft_size: u32) {
        unsafe { splt_set_fft_size(self.plot, fft_size) }
    }
    pub fn set_fft_window(&self, fft_window_name: u8) {
        unsafe { splt_set_fft_window(self.plot, fft_window_name) }
    }
    pub fn set_layout_direction(&self, direction: u8) {
        unsafe { splt_set_layout_direction(self.plot, direction) }
    }
    pub fn set_layout_histo_width(&self, histo_width: u32) {
        unsafe { splt_set_layout_histo_width(self.plot, histo_width) }
    }
    pub fn set_layout_deci_height(&self, deci_height: u32) {
        unsafe { splt_set_layout_deci_height(self.plot, deci_height) }
    }
    pub fn set_layout_ask_height(&self, ask_height: u32) {
        unsafe { splt_set_layout_ask_height(self.plot, ask_height) }
    }

    pub fn infos(&self) -> Vec<String> {
        // 'File name', value: strip(file.name) })
        // 'File type', value: file.type || 'n/a' })
        // 'File size', value: `${file.size} bytes` })
        // 'Last modified', value: lastModified.toISOString() })
        // 'Sample format', value: this.sampleFormat })
        // 'No. of samples', value: `${sampleCount} S` })
        // 'Stride (window to window)', value: `× ${stride}` })
        // 'Center frequency', value: `${this.center_freq / base_scale.scale}${base_scale.prefix}` })
        // 'Sample rate', value: `${this.sample_rate / rate_scale.scale}${rate_scale.prefix}` })
        // 'Length (time)', value: `${(sampleCount / this.sample_rate).toFixed(3)} s` })
        // 'dBfs scale', value: `${this.dBfs_min.toFixed(1)} dB – ${this.dBfs_max.toFixed(1)} dB` })

        vec![
            format!("{}", SAMPLE_FORMAT[self.sample_format() as usize]),
            format!("{:.6} MHz", self.center_freq() / 1000000.0),
            format!("{:.3} kHz", self.sample_rate() / 1000.0),
            format!("1px = {} smps", self.zoom()),
            format!("{} S", self.sample_count()),
            format!("{:.3} s", self.sample_count() as f64 / self.sample_rate()),
            format!("-{}+{} dBFS", self.db_range(), self.db_gain()),
            format!("FFT {}", self.fft_size()),
        ]
    }

    pub fn to_bitmap(&self, width: usize, height: usize, zoom: usize) -> (Vec<u8>, usize, usize) {
        //println!("Requested size: {} x {}", width, height);

        // Setup Spectroplot
        unsafe {
            splt_set_layout_size(self.plot, width as u32, height as u32);
            splt_set_zoom(self.plot, zoom as u32);
        }

        let width = unsafe { splt_get_layout_width(self.plot) } as usize;
        let height = unsafe { splt_get_layout_height(self.plot) } as usize;

        let mut pixels = vec![0; width * height];

        // Run Spectroplot
        unsafe {
            splt_draw(self.plot, pixels.as_mut_ptr(), width as u32, height as u32);
        }

        //println!("Rendered size: {} x {}", width, height);

        Self::pixels_toraw(pixels, width, height)
    }

    fn pixels_toraw(pixels: Vec<u32>, width: usize, height: usize) -> (Vec<u8>, usize, usize) {
        let mut pixels = pixels;
        (
            unsafe {
                let length = pixels.len() * 4;
                let capacity = pixels.capacity() * 4;
                let ptr = pixels.as_mut_ptr() as *mut u8;

                // Don't run the destructor for pixels
                std::mem::forget(pixels);

                // Construct new Vec
                Vec::from_raw_parts(ptr, length, capacity)
            },
            width,
            height,
        )
    }
}
