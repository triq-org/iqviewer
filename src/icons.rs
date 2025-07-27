#![allow(unused)]

use iced::Font;
use iced::widget::{Text, text};

const ICON_FONT: Font = Font::with_name("icons");
pub const FONT: &[u8] = include_bytes!("../fonts/icons.ttf");

fn icon<'a>(codepoint: char) -> Text<'a> {
    text(codepoint).font(ICON_FONT)
}

pub fn bookmark<'a>() -> Text<'a> {
    icon('\u{F097}')
}

pub fn clear<'a>() -> Text<'a> {
    icon('\u{2715}')
}

pub fn clock<'a>() -> Text<'a> {
    icon('\u{1F554}')
}

pub fn drive<'a>() -> Text<'a> {
    icon('\u{E755}')
}

pub fn file<'a>() -> Text<'a> {
    icon('\u{1F4C4}')
}

pub fn folder<'a>() -> Text<'a> {
    icon('\u{F115}')
}

pub fn gauge<'a>() -> Text<'a> {
    icon('\u{E7A2}')
}

pub fn github<'a>() -> Text<'a> {
    icon('\u{F300}')
}

pub fn grid<'a>() -> Text<'a> {
    icon('\u{268F}')
}

pub fn help<'a>() -> Text<'a> {
    icon('\u{F128}')
}

pub fn home<'a>() -> Text<'a> {
    icon('\u{2302}')
}

pub fn resize_full<'a>() -> Text<'a> {
    icon('\u{E744}')
}

pub fn resize_horizontal<'a>() -> Text<'a> {
    icon('\u{2B0D}')
}

pub fn resize_small<'a>() -> Text<'a> {
    icon('\u{E744}')
}

pub fn signal<'a>() -> Text<'a> {
    icon('\u{1F4F6}')
}

pub fn trash<'a>() -> Text<'a> {
    icon('\u{E729}')
}
