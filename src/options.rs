// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- Spectrogram options.

/// FFT window size.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FftSize {
    #[default]
    Size512,
    Size256,
    Size128,
    Size64,
    Size32,
    Size16,
}

impl FftSize {
    pub const VARIANTS: &[Self] = &[
        Self::Size512,
        Self::Size256,
        Self::Size128,
        Self::Size64,
        Self::Size32,
        Self::Size16,
    ];

    pub fn to_value(&self) -> usize {
        1 << (9 - *self as usize)
    }
}

impl std::fmt::Display for FftSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Size512 => "N=512",
            Self::Size256 => "N=256",
            Self::Size128 => "N=128",
            Self::Size64 => "N=64",
            Self::Size32 => "N=32",
            Self::Size16 => "N=16",
        })
    }
}

/// Gain value adjustment.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbGain {
    Gain0,
    Gain3,
    #[default]
    Gain6,
    Gain9,
    Gain12,
    Gain15,
    Gain18,
    Gain21,
    Gain24,
    Gain27,
    Gain30,
}

impl DbGain {
    pub const VARIANTS: &[Self] = &[
        Self::Gain0,
        Self::Gain3,
        Self::Gain6,
        Self::Gain9,
        Self::Gain12,
        Self::Gain15,
        Self::Gain18,
        Self::Gain21,
        Self::Gain24,
        Self::Gain27,
        Self::Gain30,
    ];

    pub fn to_value(&self) -> f32 {
        (3 * *self as usize) as f32
    }
}

impl std::fmt::Display for DbGain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Gain0 => "+0 dB",
            Self::Gain3 => "+3 dB",
            Self::Gain6 => "+6 dB",
            Self::Gain9 => "+9 dB",
            Self::Gain12 => "+12 dB",
            Self::Gain15 => "+15 dB",
            Self::Gain18 => "+18 dB",
            Self::Gain21 => "+21 dB",
            Self::Gain24 => "+24 dB",
            Self::Gain27 => "+27 dB",
            Self::Gain30 => "+30 dB",
        })
    }
}

/// Range value adjustment.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbRange {
    Range6,
    Range12,
    Range18,
    Range24,
    #[default]
    Range30,
    Range36,
    Range42,
    Range48,
    Range54,
    Range60,
    Range66,
    Range72,
    Range78,
    Range84,
    Range90,
}

impl DbRange {
    pub const VARIANTS: &[Self] = &[
        Self::Range6,
        Self::Range12,
        Self::Range18,
        Self::Range24,
        Self::Range30,
        Self::Range36,
        Self::Range42,
        Self::Range48,
        Self::Range54,
        Self::Range60,
        Self::Range66,
        Self::Range72,
        Self::Range78,
        Self::Range84,
        Self::Range90,
    ];

    pub fn to_value(&self) -> f32 {
        (6 * (1 + *self as usize)) as f32
    }
}

impl std::fmt::Display for DbRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Range6 => "6 dB",
            Self::Range12 => "12 dB",
            Self::Range18 => "18 dB",
            Self::Range24 => "24 dB",
            Self::Range30 => "30 dB",
            Self::Range36 => "36 dB",
            Self::Range42 => "42 dB",
            Self::Range48 => "48 dB",
            Self::Range54 => "54 dB",
            Self::Range60 => "60 dB",
            Self::Range66 => "66 dB",
            Self::Range72 => "72 dB",
            Self::Range78 => "78 dB",
            Self::Range84 => "84 dB",
            Self::Range90 => "90 dB",
        })
    }
}

/// Colormap for signal strength.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Colormap {
    #[default]
    Cube1,
    Viridis,
    Plasma,
    Inferno,
    Magma,
    Hot,
    Afmhot,
    GistHeat,
    Turbo,
    Parabola,
    Grayscale,
    Roentgen,
    Phosphor,
}

impl Colormap {
    pub const VARIANTS: &[Self] = &[
        Self::Cube1,
        Self::Viridis,
        Self::Plasma,
        Self::Inferno,
        Self::Magma,
        Self::Hot,
        Self::Afmhot,
        Self::GistHeat,
        Self::Turbo,
        Self::Parabola,
        Self::Grayscale,
        Self::Roentgen,
        Self::Phosphor,
    ];

    pub fn to_value(&self) -> usize {
        *self as usize
    }
}

impl std::fmt::Display for Colormap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Cube1 => "Cube1",
            Self::Viridis => "Viridis",
            Self::Plasma => "Plasma",
            Self::Inferno => "Inferno",
            Self::Magma => "Magma",
            Self::Hot => "Hot",
            Self::Afmhot => "Afm-Hot",
            Self::GistHeat => "Gist-Heat",
            Self::Turbo => "Turbo",
            Self::Parabola => "Parabola",
            Self::Grayscale => "Grayscale",
            Self::Roentgen => "Roentgen",
            Self::Phosphor => "Phosphor",
        })
    }
}

/// Window function for sampling.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowFunctions {
    Rectangular,
    Bartlett,
    Hann,
    Hamming,
    Blackman,
    #[default]
    BlackmanHarris,
    BlackmanNuttall,
    FlatTop,
    BartlettHann,
    Cosine,
    Lanczos,
    Gaussian,
}

impl WindowFunctions {
    pub const VARIANTS: &[Self] = &[
        Self::Rectangular,
        Self::Bartlett,
        Self::Hann,
        Self::Hamming,
        Self::Blackman,
        Self::BlackmanHarris,
        Self::BlackmanNuttall,
        Self::FlatTop,
        Self::BartlettHann,
        Self::Cosine,
        Self::Lanczos,
        Self::Gaussian,
    ];

    pub fn to_value(&self) -> usize {
        *self as usize
    }
}

impl std::fmt::Display for WindowFunctions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Rectangular => "Rectangular",
            Self::Bartlett => "Bartlett",
            Self::Hann => "Hann",
            Self::Hamming => "Hamming",
            Self::Blackman => "Blackman",
            Self::BlackmanHarris => "Blackman-Harris",
            Self::BlackmanNuttall => "Blackman-Nuttall",
            Self::FlatTop => "Flat-Top",
            Self::BartlettHann => "Bartlett-Hann",
            Self::Cosine => "Cosine",
            Self::Lanczos => "Lanczos",
            Self::Gaussian => "Gaussian",
        })
    }
}

/// Display orientation.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Spectrogram,
    Waterfall,
    Geyser,
}

impl Orientation {
    #[rustfmt::skip]
    pub const VARIANTS: &[Self] = &[
        Self::Spectrogram,
        Self::Waterfall,
        Self::Geyser,
    ];

    pub fn to_value(&self) -> usize {
        *self as usize
    }
}

impl std::fmt::Display for Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Spectrogram => "Spectrogram",
            Self::Waterfall => "Waterfall",
            Self::Geyser => "Geyser",
        })
    }
}
