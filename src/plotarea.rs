// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- Plot widget.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget, tree};
use iced::advanced::{image, renderer};
use iced::{Element, Length, Point, Rectangle, Rotation, Size, border, mouse};

use crate::plot_ffi::*;

/// Plotarea renders a Plot as raster graphics in the appropriate size.
pub struct Plotarea<'a> {
    plot: &'a Plot,
    cursor: Point,
    marker: PlotMarker,
}

impl<'a> Plotarea<'a> {
    /// Creates a plain [`Plotarea`].
    pub fn new(plot: &'a Plot) -> Self {
        Self {
            plot,
            cursor: Point::default(),
            marker: PlotMarker::default(),
        }
    }

    /// Sets the marker in the [`Plotarea`].
    pub fn marker(mut self, marker: PlotMarker) -> Self {
        self.marker = marker;
        self
    }

    /// Sets the cursor in the [`Plotarea`].
    pub fn cursor(mut self, point: Point) -> Self {
        self.cursor = point;
        self
    }
}

/// Creates a new [`Plotarea`] with the given image `Plot`.
pub fn plotarea(plot: &Plot) -> Plotarea<'_> {
    Plotarea::new(plot)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Plotarea<'a>
where
    Renderer: image::Renderer<Handle = image::Handle>,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        //let size = self.size();
        //let limits = limits.width(size.width).height(size.height);
        let limits = limits.width(Length::Fill).height(Length::Fill);
        let available = limits.max();

        let state = tree.state.downcast_mut::<State>();

        let plot_needs_redraw = self.plot.needs_redraw() || state.plot_size != available;

        if plot_needs_redraw {
            let plot_size = available.clone();
            let bitmap = self.plot.to_bitmap(
                available.width as usize,
                available.height as usize,
            );
            let plot_handle =
                image::Handle::from_rgba(bitmap.width as u32, bitmap.height as u32, bitmap.pixels);
            state.set_plot(plot_size, plot_handle);
        }

        let guide_needs_redraw = plot_needs_redraw || state.guide_cursor != self.cursor || state.guide_marker != self.marker;

        if guide_needs_redraw {
            let guide_cursor = self.cursor.clone();
            let guide_marker = self.marker.clone();
            let bitmap =
                self.plot
                    .to_guides_bitmap(self.marker, self.cursor.x as usize, self.cursor.y as usize);
            let guide_handle =
                image::Handle::from_rgba(bitmap.width as u32, bitmap.height as u32, bitmap.pixels);
            state.set_guide(guide_cursor, guide_marker, guide_handle);
        }

        layout::Node::new(Size::new(available.width, available.height))
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();

        let bounds = layout.bounds();
        let clip_bounds = Rectangle::INFINITE;

        let _allocation = renderer.load_image(&state.plot_handle).unwrap();
        renderer.draw_image(
            image::Image {
                handle: state.plot_handle.clone(),
                filter_method: image::FilterMethod::Nearest,
                rotation: Rotation::default().radians(),
                border_radius: border::Radius::default(),
                opacity: 1.0,
                snap: true,
            },
            bounds,
            clip_bounds,
        );

        let _allocation = renderer.load_image(&state.guide_handle).unwrap();
        renderer.draw_image(
            image::Image {
                handle: state.guide_handle.clone(),
                filter_method: image::FilterMethod::Nearest,
                rotation: Rotation::default().radians(),
                border_radius: border::Radius::default(),
                opacity: 1.0,
                snap: true,
            },
            bounds,
            clip_bounds,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Plotarea<'a>> for Element<'a, Message, Theme, Renderer>
where
    Renderer: image::Renderer<Handle = image::Handle>,
{
    fn from(plotarea: Plotarea<'a>) -> Self {
        Self::new(plotarea)
    }
}

/// The local state of a [`Plotarea`].
#[derive(Debug)]
pub struct State {
    plot_size: Size,
    plot_handle: image::Handle,
    guide_cursor: Point,
    guide_marker: PlotMarker,
    guide_handle: image::Handle,
}

impl Default for State {
    fn default() -> Self {
        Self {
            plot_size: Default::default(),
            plot_handle: image::Handle::from_rgba(0, 0, vec![]),
            guide_cursor: Default::default(),
            guide_marker: Default::default(),
            guide_handle: image::Handle::from_rgba(0, 0, vec![]),
        }
    }
}

impl State {
    fn set_plot(&mut self, plot_size: Size, plot_handle: image::Handle) {
        self.plot_size = plot_size;
        self.plot_handle = plot_handle;
    }

    fn set_guide(&mut self, guide_cursor: Point, guide_marker: PlotMarker, guide_handle: image::Handle) {
        self.guide_cursor = guide_cursor;
        self.guide_marker = guide_marker;
        self.guide_handle = guide_handle;
    }
}
