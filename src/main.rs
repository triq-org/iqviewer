// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- SDR I/Q data file viewer app.

// Prevent a console window on Windows
#![windows_subsystem = "windows"]

use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use std::env;
use std::path::{Path, PathBuf};
use std::usize;

use iced::mouse::ScrollDelta;
use iced::widget::scrollable::RelativeOffset;
use iced::widget::{
    Column, Container, Stack, button, column, container, grid, horizontal_space, image, pick_list,
    row, scrollable, slider, text,
};
use iced::{
    Alignment, Center, Element, Event, Length, Point, Subscription, Task, Theme, event, keyboard,
    mouse, window,
};

mod dirs;
mod icons;
mod items;
mod mouse_area;
mod options;
mod plot_ffi;
mod plotarea;

use dirs::*;
use items::*;
use mouse_area::*;
use options::*;
use plot_ffi::*;
use plotarea::*;

pub fn main() -> iced::Result {
    iced::application(Viewer::default, Viewer::update, Viewer::view)
        .subscription(Viewer::subscription)
        .title(Viewer::TITLE)
        .theme(Viewer::theme)
        .settings(Viewer::settings())
        .window(Viewer::window_settings())
        .font(icons::FONT)
        .run()
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GRID_SPACING: f32 = 10.0;
const GRID_TEXT_HEIGHT: f32 = 40.0;

//#[derive(Default)]
struct Viewer {
    screen: Screen,
    zoom_editor: bool,
    show_help: bool,
    cells_per_row: usize,
    thumbnail_size: u32,
    hover_count: usize,
    opts_fftn: Option<FftSize>,
    opts_windowf: Option<WindowFunctions>,
    opts_gain: Option<DbGain>,
    opts_range: Option<DbRange>,
    opts_colormap: Option<Colormap>,
    opts_orientation: Option<Orientation>,
    cwd: Option<PathBuf>,
    thumbnails: ItemList,
    in_click: bool,
    clicked_sample: u64,
    plot: Option<Plot>,
    is_shift_pressed: bool,
    cursor: Point,
    marker: PlotMarker,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    #[default]
    Gallery,
    Editor,
}

impl Default for Viewer {
    fn default() -> Self {
        let mut thumbnails = ItemList::default();
        thumbnails.extend(env::args().skip(1).map(|arg| PathBuf::from(arg)));

        Self {
            screen: Screen::default(),
            zoom_editor: false,
            show_help: false,
            cells_per_row: 1,
            thumbnail_size: 256,
            hover_count: 0,
            opts_fftn: Some(FftSize::default()), // FFT window width
            opts_windowf: Some(WindowFunctions::default()), // FFT windowing function
            opts_gain: Some(DbGain::default()),  // Overall gain (signal amplification)
            opts_range: Some(DbRange::default()), // Gain range (cut-off to black)
            opts_colormap: Some(Colormap::default()), // Color map
            opts_orientation: Some(Orientation::default()), // Display orientation
            cwd: None,
            thumbnails,
            in_click: false,
            clicked_sample: 0,
            plot: None,
            is_shift_pressed: false,
            cursor: Point::default(),
            marker: PlotMarker::default(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    ShowHelp,
    Quit,
    CloseEditor,
    ToggleGallery,
    ToggleSplit,
    ThumbnailSize(f32),
    GalleryScrolled(scrollable::Viewport),
    RemoveSelected,
    ClearGallery,
    OpenThumbnail(usize),
    OpenDirDialog,
    OpenFileDialog,
    FilesSelected(Option<Vec<PathBuf>>),
    FileHovered,
    FilesHoveredLeft,
    FileDropped(PathBuf),
    ToggleMark,
    ToggleDelete,
    ConfirmMove,
    ConfirmDelete,
    MoveFiles(Option<PathBuf>),
    DeleteFiles(MessageDialogResult),
    SelectPrev,
    SelectNext,
    SelectUp,
    SelectDown,
    SelectHome,
    SelectEnd,
    IncrementZoom,
    DecrementZoom,
    ResetZoom,
    PickFftn(FftSize),
    PickWindowf(WindowFunctions),
    PickGain(DbGain),
    PickRange(DbRange),
    PickColormap(Colormap),
    PickOrientation(Orientation),
    PlotLeftPress(Point),
    PlotMove(Point),
    PlotLeftRelease(Point),
    PlotMiddlePress(Point),
    PlotRightPress(Point),
    PlotDoubleClicked,
    PlotScroll(Point, ScrollDelta),
    ShiftPressed,
    ShiftReleased,
}

impl Viewer {
    const TITLE: &'static str = "I/Q Viewer";

    fn theme(&self) -> Theme {
        Theme::CatppuccinFrappe
    }

    fn settings() -> iced::Settings {
        iced::Settings {
            id: Some("org.triq.iqviewer".to_string()),
            ..Default::default()
        }
    }

    fn window_settings() -> window::Settings {
        let rgba = include_bytes!("../assets/icon.rgba");
        let icon = window::icon::from_rgba(rgba.to_vec(), 512, 512).expect("Bad Icon data");
        let platform_specific = window::settings::PlatformSpecific::default();
        #[cfg(target_os = "linux")]
        let platform_specific = window::settings::PlatformSpecific {
            application_id: "IQViewer".to_owned(),
            ..Default::default()
        };
        window::Settings {
            icon: Some(icon),
            min_size: Some((400.0, 400.0).into()),
            platform_specific,
            ..Default::default()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            keyboard::on_key_press(Self::on_key_press),
            keyboard::on_key_release(Self::on_key_release),
            event::listen_with(|event, _status, _windows| match event {
                Event::Window(window::Event::FileHovered(_path)) => Some(Message::FileHovered),
                Event::Window(window::Event::FilesHoveredLeft) => Some(Message::FilesHoveredLeft),
                Event::Window(window::Event::FileDropped(path)) => Some(Message::FileDropped(path)),
                _ => None,
            }),
        ])
    }

    fn on_key_press(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
        use keyboard::Key::{Character, Named};
        use keyboard::key::Named as Key;

        const NONE: keyboard::Modifiers = keyboard::Modifiers::from_bits(0).unwrap();
        const SHIFT: keyboard::Modifiers = keyboard::Modifiers::SHIFT;

        match (key.as_ref(), modifiers) {
            (Named(Key::Shift), _) => Some(Message::ShiftPressed),
            (Named(Key::ArrowLeft), NONE) => Some(Message::SelectPrev),
            (Named(Key::ArrowRight), NONE) => Some(Message::SelectNext),
            (Named(Key::ArrowUp), NONE) => Some(Message::SelectUp),
            (Named(Key::ArrowDown), NONE) => Some(Message::SelectDown),
            //(Named(Key::PageUp), NONE) => Some(Message::SelectPageUp),
            //(Named(Key::PageDown), NONE) => Some(Message::SelectPageDown),
            (Named(Key::Home), NONE) => Some(Message::SelectHome),
            (Named(Key::End), NONE) => Some(Message::SelectEnd),
            (Named(Key::Escape), NONE) => Some(Message::CloseEditor),
            (Named(Key::Space), NONE) => Some(Message::ToggleGallery),
            (Named(Key::Delete), NONE) => Some(Message::RemoveSelected),
            (Character("d"), SHIFT) => Some(Message::ConfirmDelete),
            (Character("m"), SHIFT) => Some(Message::ConfirmMove),
            (Character("o"), SHIFT) => Some(Message::OpenDirDialog),
            (Character("o"), NONE) => Some(Message::OpenFileDialog),
            (Character("x"), NONE) => Some(Message::ClearGallery),
            (Character("q"), NONE) => Some(Message::Quit),
            (Character("f"), NONE) => Some(Message::ToggleMark),
            (Character("d"), NONE) => Some(Message::ToggleDelete),
            (Character("s"), NONE) => Some(Message::ThumbnailSize(64.0)),
            (Character("m"), NONE) => Some(Message::ThumbnailSize(128.0)),
            (Character("l"), NONE) => Some(Message::ThumbnailSize(256.0)),
            (Character("+"), NONE) => Some(Message::IncrementZoom),
            (Character("-"), NONE) => Some(Message::DecrementZoom),
            (Character("0"), NONE) => Some(Message::ResetZoom),
            (Character("z"), NONE) => Some(Message::ToggleSplit),
            (Character("h"), NONE) => Some(Message::ShowHelp),
            _ => None,
        }
    }

    fn on_key_release(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
        use keyboard::Key::Named;
        use keyboard::key::Named as Key;

        match (key.as_ref(), modifiers) {
            (Named(Key::Shift), _) => Some(Message::ShiftReleased),
            _ => None,
        }
    }

    async fn open_dir_dialog() -> Option<Vec<PathBuf>> {
        // TODO: AsyncFileDialog::new() ?
        FileDialog::new()
            .set_title("Open I/Q files folder")
            .pick_folders()
    }

    async fn open_files_dialog() -> Option<Vec<PathBuf>> {
        // TODO: AsyncFileDialog::new() ?
        FileDialog::new()
            .set_title("Open I/Q data files")
            .add_filter("I/Q Sample", dirs::FORMATS)
            .pick_files()
    }

    async fn save_dir_dialog() -> Option<PathBuf> {
        // TODO: AsyncFileDialog::new() ?
        FileDialog::new()
            .set_title("Move marked files")
            .set_can_create_directories(true)
            .pick_folder()
    }

    async fn confirm_delete_dialog(count: usize) -> MessageDialogResult {
        // TODO: AsyncMessageDialog::new() ?
        let description = format!("Do you want to delete {} files?", count);
        MessageDialog::new()
            .set_buttons(MessageButtons::OkCancel)
            .set_description(description)
            .set_level(MessageLevel::Warning)
            .set_title("Delete files?")
            .show()
    }

    /// Quick hack to get cells_per_row for a grid.
    fn thumbnails_scroll_position(&self) -> f32 {
        // get row postion
        let cells_per_row = self.cells_per_row.max(1);
        let total_rows = (self.thumbnails.len() + cells_per_row - 1) / cells_per_row;
        let selection_row = (self.thumbnails.selection()) / cells_per_row;
        let visible_rows = 1; // TODO: compute from height and item size?
        let y = selection_row as f32 / (total_rows.max(visible_rows + 1) - visible_rows) as f32;
        //println!("total_rows {}, thumbnails.len {} cells_per_row {} selection_row {} self.selection {} y {}",
        //    total_rows, self.thumbnails.len(), cells_per_row, selection_row, self.selection, y);
        y
    }

    fn open_plot(&mut self, path: impl AsRef<Path>) {
        let path = path.as_ref();
        if self.plot.is_none() {
            let plot = Plot::with_path(path);
            self.plot = Some(plot);
        } else {
            self.plot.as_mut().unwrap().open(path);
        }
        // Apply all settings
        if let Some(plot) = self.plot.as_ref() {
            plot.set_cmap(self.opts_colormap.unwrap_or_default().to_value() as u32);
            plot.set_fft_size(self.opts_fftn.unwrap_or_default().to_value() as u32);
            plot.set_fft_window(self.opts_windowf.unwrap_or_default().to_value() as u8);
            plot.set_db_gain(self.opts_gain.unwrap_or_default().to_value());
            plot.set_db_range(self.opts_range.unwrap_or_default().to_value());
            plot.set_cmap(self.opts_colormap.unwrap_or_default().to_value() as u32);
            plot.set_layout_direction(self.opts_orientation.unwrap_or_default().to_value() as u8);
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Quit => return window::get_latest().and_then(window::close),
            Message::ShowHelp => {
                self.show_help = !self.show_help;
            }
            Message::CloseEditor => {
                if self.show_help {
                    // Close help if it's open
                    self.show_help = !self.show_help;
                } else {
                    // Otherwise close Editor, return to gallery
                    self.screen = Screen::Gallery
                }
            }
            Message::ToggleGallery => {
                if self.thumbnails.is_empty() {
                    // do nothing
                } else if self.screen == Screen::Editor {
                    let thumbnail = self.thumbnails.selected().unwrap();
                    if thumbnail.path() == self.plot.as_ref().unwrap().path() {
                        self.screen = Screen::Gallery
                    } else {
                        let path = thumbnail.path();
                        self.open_plot(path.to_path_buf());
                    }
                } else {
                    let thumbnail = self.thumbnails.selected().unwrap();
                    let path = thumbnail.path();
                    self.open_plot(path.to_path_buf());
                    self.screen = Screen::Editor
                }
            }
            Message::ToggleSplit => {
                self.zoom_editor = !self.zoom_editor;
            }
            Message::GalleryScrolled(viewport) => {
                // TODO: save/restore offset
                // println!("relative_offset {:?}", viewport.relative_offset().y);
                let scrollable_available_width = viewport.content_bounds().width;
                let max_width = self.thumbnail_size;
                // width = n * (cell + spacing) - spacing, given n > 0
                self.cells_per_row = ((scrollable_available_width + GRID_SPACING)
                    / (max_width as f32 + GRID_SPACING))
                    .ceil() as usize;

                // TODO: calculate row count
                // let thumbnail_available_width = scrollable_available_width - GRID_SPACING * (self.cells_per_row + 1) as f32;
                // let thumbnail_width = thumbnail_available_width / self.cells_per_row as f32;
                // let thumbnail_scale = thumbnail_width / self.thumbnail_size as f32;
                // let thumbnail_height = (self.thumbnail_size as f32 + GRID_TEXT_HEIGHT) * thumbnail_scale;
                // let scrollable_visible_height = viewport.bounds().height;
                // let visible_rows = scrollable_visible_height / (thumbnail_height + GRID_SPACING);
                // println!("rows {visible_rows} width {thumbnail_width} scale {thumbnail_scale}  viewport.bounds() {:?}", viewport.bounds());
            }
            Message::ThumbnailSize(size) => {
                self.thumbnail_size = size as u32;
            }
            Message::ClearGallery => {
                self.screen = Screen::Gallery;
                self.thumbnails.clear();
            }
            Message::RemoveSelected => {
                self.thumbnails.selected_remove();
            }
            Message::ToggleMark => {
                self.thumbnails.selected_toggle_mark();
            }
            Message::ToggleDelete => {
                self.thumbnails.selected_toggle_delete();
            }
            Message::ConfirmMove => {
                if self.thumbnails.count_marked() > 0 {
                    return Task::perform(Self::save_dir_dialog(), Message::MoveFiles);
                }
            }
            Message::ConfirmDelete => {
                if self.thumbnails.count_to_delete() > 0 {
                    return Task::perform(
                        Self::confirm_delete_dialog(self.thumbnails.count_to_delete()),
                        Message::DeleteFiles,
                    );
                }
            }
            Message::MoveFiles(path) => {
                if let Some(path) = path {
                    self.thumbnails.move_marked_to(path);
                }
            }
            Message::DeleteFiles(dialog_result) => {
                if dialog_result == MessageDialogResult::Ok {
                    self.thumbnails.delete_marked();
                }
            }
            Message::OpenThumbnail(index) => {
                if self.thumbnails.selection() == index {
                    let path = self.thumbnails.selected().unwrap().path();
                    self.open_plot(path.to_path_buf());
                    self.screen = Screen::Editor
                } else {
                    self.thumbnails.set_selection(index);
                }
            }
            Message::OpenDirDialog => {
                return Task::perform(Self::open_dir_dialog(), Message::FilesSelected);
            }
            Message::OpenFileDialog => {
                return Task::perform(Self::open_files_dialog(), Message::FilesSelected);
            }
            Message::FilesSelected(files) => {
                if let Some(files) = files {
                    if !files.is_empty() {
                        // println!("FilesSelected {:?}", files);
                        let first = files.first().unwrap();
                        if first.is_file() {
                            self.open_plot(first);
                        } else {
                            self.cwd = files.first().cloned();
                        }
                    }

                    self.thumbnails.extend(files);
                }
            }
            Message::FileHovered => self.hover_count += 1,
            Message::FilesHoveredLeft => self.hover_count = 0,
            Message::FileDropped(path) => {
                // println!("FileDropped (of {}) {:?}", self.hover_count, path);
                if path.is_file() {
                    if self.hover_count == 1 {
                        // single file: open editor
                        self.screen = Screen::Editor;
                        self.hover_count = 0;
                    } else if self.hover_count > 1 {
                        // multiple files: close editor
                        self.screen = Screen::Gallery;
                        self.hover_count = 0;
                    }

                    self.thumbnails.push(path.clone());
                    self.open_plot(&path);
                } else {
                    // dir of files: close editor
                    self.screen = Screen::Gallery;

                    self.cwd = Some(path.clone());

                    let files = read_dir_iq(path).unwrap();
                    self.thumbnails.extend(files);
                }
            }
            Message::SelectPrev => {
                self.thumbnails.dec_selection(1);
                let y = self.thumbnails_scroll_position();
                return scrollable::snap_to("gallery", RelativeOffset { x: 0.0, y });
            }
            Message::SelectNext => {
                self.thumbnails.inc_selection(1);
                let y = self.thumbnails_scroll_position();
                return scrollable::snap_to("gallery", RelativeOffset { x: 0.0, y });
            }
            Message::SelectUp => {
                self.thumbnails.dec_selection(self.cells_per_row);
                let y = self.thumbnails_scroll_position();
                return scrollable::snap_to("gallery", RelativeOffset { x: 0.0, y });
            }
            Message::SelectDown => {
                self.thumbnails.inc_selection(self.cells_per_row);
                let y = self.thumbnails_scroll_position();
                return scrollable::snap_to("gallery", RelativeOffset { x: 0.0, y });
            }
            Message::SelectHome => {
                self.thumbnails.set_selection(0);
                return scrollable::snap_to(
                    "gallery",
                    scrollable::RelativeOffset { x: 0.0, y: 0.0 },
                );
            }
            Message::SelectEnd => {
                self.thumbnails.set_selection(usize::MAX);
                return scrollable::snap_to(
                    "gallery",
                    scrollable::RelativeOffset { x: 0.0, y: 1.0 },
                );
            }
            Message::IncrementZoom => {
                if let Some(plot) = self.plot.as_mut() {
                    let x = plot.width() / 2; // NOTE: zoom at roughly center
                    let y = plot.height() / 2;
                    plot.set_zoom_at(x, y, (plot.zoom() / 2).max(1));
                }
            }
            Message::DecrementZoom => {
                if let Some(plot) = self.plot.as_mut() {
                    let x = plot.width() / 2; // NOTE: zoom at roughly center
                    let y = plot.height() / 2;
                    plot.set_zoom_at(x, y, plot.zoom() * 2);
                }
            }
            Message::ResetZoom => {
                if let Some(plot) = self.plot.as_mut() {
                    plot.set_zoom(0);
                }
            }
            Message::PickFftn(val) => {
                self.opts_fftn = Some(val);
                self.plot
                    .as_ref()
                    .unwrap()
                    .set_fft_size(val.to_value() as u32);
            }
            Message::PickWindowf(val) => {
                self.opts_windowf = Some(val);
                self.plot
                    .as_ref()
                    .unwrap()
                    .set_fft_window(val.to_value() as u8);
            }
            Message::PickGain(val) => {
                self.opts_gain = Some(val);
                self.plot.as_ref().unwrap().set_db_gain(val.to_value());
            }
            Message::PickRange(val) => {
                self.opts_range = Some(val);
                self.plot.as_ref().unwrap().set_db_range(val.to_value());
            }
            Message::PickColormap(val) => {
                self.opts_colormap = Some(val);
                self.plot.as_ref().unwrap().set_cmap(val.to_value() as u32);
            }
            Message::PickOrientation(val) => {
                self.opts_orientation = Some(val);
                self.plot
                    .as_ref()
                    .unwrap()
                    .set_layout_direction(val.to_value() as u8);
            }
            Message::PlotLeftPress(position) => {
                if let Some(plot) = self.plot.as_mut() {
                    if self.is_shift_pressed {
                        if self.marker.sample != 0
                            && plot.is_nearby(
                                self.marker.sample,
                                self.marker.freq,
                                position.x as u32,
                                position.y as u32,
                            )
                        {
                            // remove marker
                            self.marker = PlotMarker::default();
                        } else {
                            // toggle marker
                            self.marker.sample =
                                plot.sample_at_pos(position.x as u32, position.y as u32);
                            self.marker.freq =
                                plot.freq_at_pos(position.x as u32, position.y as u32);
                        }
                    } else {
                        // pan view
                        self.clicked_sample =
                            plot.sample_at_pos(position.x as u32, position.y as u32);
                        self.in_click = true;
                    }
                }
            }
            Message::PlotMove(position) => {
                self.cursor = position;
                if self.in_click {
                    if let Some(plot) = self.plot.as_mut() {
                        plot.pan_to_pos(self.clicked_sample, position.x as u32, position.y as u32);
                    }
                }
            }
            Message::PlotLeftRelease(position) => {
                if self.in_click {
                    if let Some(plot) = self.plot.as_mut() {
                        plot.pan_to_pos(self.clicked_sample, position.x as u32, position.y as u32);
                    }
                    self.in_click = false;
                }
            }
            Message::PlotMiddlePress(position) => {
                if let Some(plot) = self.plot.as_mut() {
                    plot.set_zoom_at(
                        position.x as u32,
                        position.y as u32,
                        (plot.zoom() / 2).max(1),
                    );
                }
            }
            Message::PlotRightPress(position) => {
                if let Some(plot) = self.plot.as_mut() {
                    plot.set_zoom_at(position.x as u32, position.y as u32, plot.zoom() * 2);
                }
            }
            Message::PlotDoubleClicked => {
                if let Some(plot) = self.plot.as_mut() {
                    plot.set_zoom(0);
                }
            }
            Message::PlotScroll(position, delta) => {
                let (dx, dy) = match delta {
                    ScrollDelta::Lines { x, y } => (x, y),
                    ScrollDelta::Pixels { x, y } => (x, y),
                };
                if dy > 0.0 {
                    if let Some(plot) = self.plot.as_mut() {
                        plot.set_zoom_at(
                            position.x as u32,
                            position.y as u32,
                            (plot.zoom() / 2).max(1),
                        );
                    }
                } else if dy < 0.0 {
                    if let Some(plot) = self.plot.as_mut() {
                        plot.set_zoom_at(position.x as u32, position.y as u32, plot.zoom() * 2);
                    }
                } else {
                    if let Some(plot) = self.plot.as_mut() {
                        let zoom = plot.zoom() as i32;
                        plot.set_pan_by(dx.signum() as i32 * 50 * zoom, 0);
                    }
                }
            }
            Message::ShiftPressed => self.is_shift_pressed = true,
            Message::ShiftReleased => self.is_shift_pressed = false,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = match (self.screen, self.zoom_editor) {
            (Screen::Gallery, _) => self.view_gallery().height(Length::FillPortion(1)).into(),
            (Screen::Editor, true) => self.view_editor().into(),
            (Screen::Editor, false) => column![
                self.view_gallery().height(Length::FillPortion(1)),
                self.view_editor().height(Length::FillPortion(2)),
            ]
            .align_x(Center)
            .into(),
        };

        let content = column![content, self.view_statusbar(),].into();

        if self.show_help {
            Stack::with_children([content, self.view_help().into()]).into()
        } else {
            content
        }
    }

    fn view_statusbar(&self) -> Container<Message> {
        let marked = self.thumbnails.count_marked();
        let to_delete = self.thumbnails.count_to_delete();
        let item_count = self.thumbnails.len();
        let status_text = row![
            row![icons::grid(), text(format!(" {item_count}"))],
            row![icons::bookmark(), text(format!(" {marked}"))],
            row![icons::trash(), text(format!(" {to_delete}"))],
        ]
        .spacing(16);

        let selection_text = if let Some(thumbnail) = self.thumbnails.selected() {
            let filename = thumbnail.filename();
            let size = thumbnail.size().unwrap_or_default() / 1024;
            let sample_format = thumbnail.sample_format();
            let sample_count = thumbnail.sample_count();
            let center_freq = thumbnail.center_freq() / 1_000_000.0;
            let sample_rate = thumbnail.sample_rate() / 1_000.0;
            let duration = thumbnail.sample_count() as f64 / thumbnail.sample_rate();
            row![
                row![icons::file(), text(format!(" {filename}"))],
                row![icons::drive(), text(format!(" {size} kB  {sample_format}"))],
                row![icons::resize_horizontal(), text(format!(" {sample_count} S"))],
                row![icons::clock(), text(format!(" {duration:.2} s"))],
                row![icons::signal(), text(format!(" {center_freq} MHz"))],
                row![icons::gauge(), text(format!(" {sample_rate} kHz"))],
            ]
            .spacing(16)
        } else {
            row![]
        };

        container(row![selection_text, status_text,].spacing(20))
            .padding([0, 10]) // top/bottom, left/right
            .width(Length::Fill)
            .style(container::rounded_box)
    }

    fn view_help(&self) -> Container<Message> {
        container(
            container(
                column![
                    text(format!("IQViewer {VERSION}"))
                        .size(20)
                        .style(text::primary),
                    text(""),
                    text("Drop files or dirs to load"),
                    text("or use menu to open files / folders"),
                    row![
                        column![
                            text("Hotkeys:"),
                            dt_text("o", "open files"),
                            dt_text("O", "open dirs"),
                            dt_text("x", "clear list"),
                            dt_text("DEL", "remove item"),
                            dt_text("d", "mark file for delete"),
                            dt_text("f", "mark file for move"),
                            dt_text("D", "delete marked"),
                            dt_text("M", "move marked"),
                            dt_text("SPACE", "toggle viewer"),
                            dt_text("z", "toggle viewer size"),
                            dt_text("q", "quit app"),
                            dt_text("s m l", "thumbnail size"),
                            dt_text("h", "toggle this help"),
                            dt_text("↑↓←→", "move selection"),
                            dt_text("⤒⤓", "move first / last"),
                        ]
                        .padding(20),
                        column![
                            text(""),
                            text("Viewer hotkeys:"),
                            dt_text("ESC", "close viewer"),
                            dt_text("SPACE", "toggle viewer"),
                            dt_text("+", "zoom in"),
                            dt_text("-", "zoom out"),
                            dt_text("0", "reset zoom"),
                            text(""),
                            text("Viewer mouse controls:"),
                            dt2_text("Scroll Wheel", "zoom"),
                            dt2_text("Horizontal Scroll", "pan"),
                            dt2_text("Click+Drag", "pan"),
                            dt2_text("Middle Click", "zoom in"),
                            dt2_text("Right Click", "zoom out"),
                            dt2_text("Hold Shift", "measure"),
                            dt2_text("Shift+Click", "set a marker"),
                        ]
                        .padding(20)
                    ],
                    row![icons::home(), "https://triq.net/"].spacing(6),
                    row![icons::github(), "https://github.com/triq-org/iqviewer/"].spacing(6),
                ]
                .align_x(Alignment::Center),
            )
            .padding(50)
            .style(container::rounded_box),
        )
        .center(Length::Fill)
    }

    fn thumbnail_style(&self, index: usize) -> fn(&Theme, button::Status) -> button::Style {
        if index == self.thumbnails.selection() {
            button::primary
        } else {
            let thumbnail = self.thumbnails.get(index).unwrap();
            if thumbnail.has_delete() {
                button::danger
            } else if thumbnail.has_mark() {
                button::success
            } else {
                button::text
            }
        }
    }

    fn thumbnail_text_style(&self, index: usize) -> fn(&Theme) -> container::Style {
        let thumbnail = self.thumbnails.get(index).unwrap();
        if thumbnail.has_delete() {
            container::danger
        } else if thumbnail.has_mark() {
            container::success
        } else {
            container::transparent
        }
    }

    fn view_thumbnails(&self) -> Container<Message> {
        //let thumbnails: Vec<iced::Element<'_, Message>> = vec![];
        let thumbnails = self.thumbnails.iter().enumerate().map(|(index, thumbnail)|
                // TODO: mouse_area for double_click?
                button(column![
                    image(thumbnail.handle())
                        .filter_method(image::FilterMethod::Nearest),
                    container(
                    text(thumbnail.filename())
                        .size(14)
                        .wrapping(text::Wrapping::Glyph),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(self.thumbnail_text_style(index))
                ])
                .on_press(Message::OpenThumbnail(index))
                .padding(2)
                //.style(button::text)
                .style(self.thumbnail_style(index))
                .into());

        let t_width = self.thumbnail_size;
        let t_height = self.thumbnail_size + GRID_TEXT_HEIGHT as u32;

        // // width = n * (cell + spacing) - spacing, given n > 0
        // let cells_per_row = ((self.scrollable_available_width + GRID_SPACING) / (t_width as f32 + GRID_SPACING)).ceil() as usize;
        // println!("{} / {} -> {}", self.scrollable_available_width, t_width, cells_per_row);

        let gallery = grid(thumbnails)
            .fluid(t_width)
            .height(grid::aspect_ratio(t_width, t_height))
            .spacing(GRID_SPACING);

        container(
            scrollable(gallery)
                .id("gallery")
                .on_scroll(Message::GalleryScrolled)
                .spacing(10),
        )
        .padding(10)
    }

    fn view_menubar(&self) -> Container<Message> {
        let menubar = row![
            button(row![icons::folder(), " Open folder"])
                .style(button::text)
                .on_press(Message::OpenDirDialog),
            button(row![icons::file(), " Open files"])
                .style(button::text)
                .on_press(Message::OpenFileDialog),
            button(row![icons::clear(), " Clear list"])
                .style(button::text)
                .on_press(Message::ClearGallery),
            button(row![icons::help(), " Help"])
                .style(button::text)
                .on_press(Message::ShowHelp),
            horizontal_space(),
            container(slider(
                64.0..=256.0,
                self.thumbnail_size as f32,
                Message::ThumbnailSize
            ),)
            .padding([7, 5]), // top/bottom, left/right
        ]
        .spacing(1);

        container(menubar)
            .padding([0, 10]) // top/bottom, left/right
            .width(Length::Fill)
            .style(container::rounded_box)
    }

    fn view_gallery(&self) -> Column<Message> {
        let content = if self.thumbnails.is_empty() {
            self.view_help()
        } else {
            self.view_thumbnails()
        };

        column![self.view_menubar(), content,].align_x(Center)
    }

    fn view_editor(&self) -> Column<Message> {
        let options_fftn =
            pick_list(FftSize::VARIANTS, self.opts_fftn, Message::PickFftn).placeholder("FFT N");

        let options_windowf = pick_list(
            WindowFunctions::VARIANTS,
            self.opts_windowf,
            Message::PickWindowf,
        )
        .placeholder("Windowing");

        let options_gain =
            pick_list(DbGain::VARIANTS, self.opts_gain, Message::PickGain).placeholder("Gain");

        let options_range =
            pick_list(DbRange::VARIANTS, self.opts_range, Message::PickRange).placeholder("Range");

        let options_colormap = pick_list(
            Colormap::VARIANTS,
            self.opts_colormap,
            Message::PickColormap,
        )
        .placeholder("Colormap");

        let options_orientation = pick_list(
            Orientation::VARIANTS,
            self.opts_orientation,
            Message::PickOrientation,
        )
        .placeholder("Orientation");

        let toolbar = row![
            column![text("FFT window width").size(12), options_fftn].align_x(Alignment::Center),
            column![text("FFT windowing function").size(12), options_windowf]
                .align_x(Alignment::Center),
            column![
                text("Overall gain" /*"(signal amplification)"*/).size(12),
                options_gain
            ]
            .align_x(Alignment::Center),
            column![
                text("Gain range" /*"(cut-off to black)"*/).size(12),
                options_range
            ]
            .align_x(Alignment::Center),
            column![text("Color map").size(12), options_colormap].align_x(Alignment::Center),
            column![text("Display orientation").size(12), options_orientation]
                .align_x(Alignment::Center),
        ]
        .wrap();
        let toolbar = container(toolbar).padding([0, 10]);

        /*
        let actionbar = row![
            button(icons::new_icon()).on_press(Message::OpenFileDialog),
            button("Zoom Out").on_press(Message::DecrementZoom),
            container(text(format!("1px = {} smps", self.zoom)).size(30)).style(container::rounded_box),
            button("Zoom In").on_press(Message::IncrementZoom),
        ]
        .spacing(5)
        .padding(5);
        */

        let infos = self.plot.as_ref().unwrap().infos();
        let infobar = infos.into_iter().map(|info| {
            container(text(info).size(14))
                .style(container::rounded_box)
                .into()
        });
        let infobar = row(infobar).spacing(5).padding([5, 10]);

        let plot = plotarea(self.plot.as_ref().unwrap())
            .marker(self.marker)
            .cursor(self.cursor);

        let plot = MouseArea::new(plot)
            .on_press(Message::PlotLeftPress)
            .on_move_maybe((self.in_click || self.is_shift_pressed).then_some(Message::PlotMove))
            .on_release(Message::PlotLeftRelease)
            .on_middle_press(Message::PlotMiddlePress)
            .on_right_press(Message::PlotRightPress)
            .on_double_click(Message::PlotDoubleClicked)
            .on_scroll(Message::PlotScroll)
            .interaction(mouse::Interaction::Crosshair);

        column![
            toolbar,
            //actionbar,
            infobar,
            plot,
        ]
    }
}

/// Definition term (DT) text, `term` is centered within 70px, definition is left aligned.
fn dt_text<'a>(term: &'a str, definition: &'a str) -> Element<'a, Message> {
    row![
        container(text(term).style(text::success)).center_x(70),
        container(text(definition)),
    ]
    .into()
}
fn dt2_text<'a>(term: &'a str, definition: &'a str) -> Element<'a, Message> {
    row![
        container(text(term).style(text::success)).center_x(150),
        container(text(definition)),
    ]
    .into()
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use iced_test::selector::text;
    use iced_test::{Error, simulator};

    #[test]
    fn it_zooms() -> Result<(), Error> {
        let mut viewer = Viewer { zoom: 0 };
        let mut ui = simulator(viewer.view());

        let _ = ui.click(text("Increment"))?;
        let _ = ui.click(text("Increment"))?;
        let _ = ui.click(text("Decrement"))?;

        for message in ui.into_messages() {
            viewer.update(message);
        }

        assert_eq!(viewer.zoom, 1);

        let mut ui = simulator(viewer.view());
        assert!(ui.find(text("1")).is_ok(), "Viewer should display 1!");

        Ok(())
    }
}
*/
