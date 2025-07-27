// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (2025) Christian W. Zuckschwerdt

//! I/Q Viewer -- Plot widget.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, Widget};
use iced::advanced::{image, renderer};
use iced::{Element, Length, Point, Rectangle, Rotation, Size, mouse};

use crate::plot_ffi::*;

pub struct Plotarea<'a> {
    plot: &'a Plot,
    cursor: Point,
    marker: PlotMarker,
}

/// Plotarea renders raster graphics in the appropriate size.
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
pub fn plotarea(plot: &Plot) -> Plotarea {
    Plotarea::new(plot)
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Plotarea<'a>
where
    Renderer: image::Renderer<Handle = image::Handle>,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        //let size = self.size();
        //let limits = limits.width(size.width).height(size.height);
        let limits = limits.width(Length::Fill).height(Length::Fill);
        let available = limits.max();

        layout::Node::new(Size::new(available.width, available.height))
    }

    fn draw(
        &self,
        _state: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let drawing_bounds = layout.bounds();

        let bitmap = self.plot.to_bitmap(
            drawing_bounds.width as usize,
            drawing_bounds.height as usize,
        );
        let handle =
            image::Handle::from_rgba(bitmap.width as u32, bitmap.height as u32, bitmap.pixels);
        renderer.draw_image(
            image::Image {
                handle: handle,
                filter_method: image::FilterMethod::Nearest,
                rotation: Rotation::default().radians(),
                opacity: 1.0,
                snap: true,
            },
            drawing_bounds,
        );

        let bitmap =
            self.plot
                .to_guides_bitmap(self.marker, self.cursor.x as usize, self.cursor.y as usize);
        let handle =
            image::Handle::from_rgba(bitmap.width as u32, bitmap.height as u32, bitmap.pixels);
        renderer.draw_image(
            image::Image {
                handle: handle,
                filter_method: image::FilterMethod::Nearest,
                rotation: Rotation::default().radians(),
                opacity: 1.0,
                snap: true,
            },
            drawing_bounds,
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
