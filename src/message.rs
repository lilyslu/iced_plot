use iced::Rectangle;

use crate::{camera::Camera, series::ShapeId, ticks::PositionedTick};

/// Messages sent by the plot widget to the application.
///
/// These messages are generated in response to user interactions with the plot.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum PlotUiMessage {
    /// Toggle the legend visibility.
    ToggleLegend,
    /// Toggle the in-canvas controls/help overlay.
    ToggleControlsOverlay,
    /// Toggle visibility of a series or reference line by label.
    ToggleSeriesVisibility(ShapeId),
    /// Internal render update message.
    RenderUpdate(PlotRenderUpdate),
}

impl PlotUiMessage {
    /// Get the hover or pick event from the render update.
    /// If the plot widget is not in hover or pick mode, this will return None.
    pub fn get_hover_pick_event(&self) -> Option<HoverPickEvent> {
        if let PlotUiMessage::RenderUpdate(update) = self {
            update.hover_pick
        } else {
            None
        }
    }

    /// Get the drag event from the render update.
    pub fn get_drag_event(&self) -> Option<DragEvent> {
        if let PlotUiMessage::RenderUpdate(update) = self {
            update.drag_event
        } else {
            None
        }
    }

    /// Get the current public plot view bounds from the render update.
    ///
    /// Returns `None` for non-render messages, or for render messages that do
    /// not carry a camera/viewport update.
    pub fn get_view_bounds(&self) -> Option<PlotViewBounds> {
        if let PlotUiMessage::RenderUpdate(update) = self {
            update.view_bounds()
        } else {
            None
        }
    }

    /// Get the current public plot view change from the render update.
    ///
    /// Returns `None` for non-render messages, or for render messages that do
    /// not carry a camera/viewport change.
    pub fn get_view_change(&self) -> Option<PlotViewChange> {
        if let PlotUiMessage::RenderUpdate(update) = self {
            update.view_change()
        } else {
            None
        }
    }
}

/// Context passed to hover/pick highlight callbacks.
///
/// Contains information identifying the point being highlighted.
#[derive(Debug, Clone, Copy)]
pub struct TooltipContext<'a> {
    /// ID of the series
    pub series_id: ShapeId,
    /// Label of the series, if any (empty string means none)
    pub series_label: &'a str,
    /// Index within the series [0..len)
    pub point_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TooltipUiPayload {
    /// screen coordinates of the tooltip.
    /// `screen_xy = None` means the tooltip is outside of the plot widget
    pub screen_xy: Option<[f32; 2]>,
    pub text: String,
}

/// Payload for the small cursor-position overlay shown in the corner.
#[derive(Debug, Clone)]
pub struct CursorPositionUiPayload {
    /// World/data-space coordinates for the cursor
    pub x: f64,
    pub y: f64,
    /// Formatted text to render
    pub text: String,
}

/// Public snapshot of the plot camera and viewport in world/data coordinates.
///
/// This intentionally exposes a stable value type instead of the internal
/// camera implementation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotViewBounds {
    /// Minimum visible x value in world/data coordinates.
    pub x_min: f64,
    /// Maximum visible x value in world/data coordinates.
    pub x_max: f64,
    /// Minimum visible y value in world/data coordinates.
    pub y_min: f64,
    /// Maximum visible y value in world/data coordinates.
    pub y_max: f64,
    /// Camera center x value in world/data coordinates.
    pub center_x: f64,
    /// Camera center y value in world/data coordinates.
    pub center_y: f64,
    /// Camera half width in world/data coordinates.
    pub half_width: f64,
    /// Camera half height in world/data coordinates.
    pub half_height: f64,
    /// Plot viewport width in screen pixels.
    pub viewport_width: f64,
    /// Plot viewport height in screen pixels.
    pub viewport_height: f64,
}

/// Public snapshot of a plot camera/viewport update.
///
/// The `bounds` field is the same view snapshot returned by
/// [`PlotUiMessage::get_view_bounds`]. The boolean fields classify the view
/// update without requiring applications to diff consecutive bounds snapshots.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotViewChange {
    pub bounds: PlotViewBounds,
    pub x_zoomed: bool,
    pub y_zoomed: bool,
    pub panned: bool,
    pub resized: bool,
    /// Whether this view update coincided with a changed per-plot x-axis link offset.
    ///
    /// This identifies the programmatic camera movement caused by
    /// [`crate::PlotWidget::set_x_axis_link_offset`]. It can be true alongside
    /// the geometric classifications above when changes are coalesced into one
    /// render update.
    pub x_axis_link_offset_changed: bool,
}

impl PlotViewBounds {
    pub(crate) fn from_camera_bounds(camera: &Camera, bounds: &Rectangle) -> Self {
        Self {
            x_min: camera.position.x - camera.half_extents.x,
            x_max: camera.position.x + camera.half_extents.x,
            y_min: camera.position.y - camera.half_extents.y,
            y_max: camera.position.y + camera.half_extents.y,
            center_x: camera.position.x,
            center_y: camera.position.y,
            half_width: camera.half_extents.x,
            half_height: camera.half_extents.y,
            viewport_width: f64::from(bounds.width),
            viewport_height: f64::from(bounds.height),
        }
    }

    /// Return the visible x range as `(min, max)`.
    pub fn x_range(self) -> (f64, f64) {
        (self.x_min, self.x_max)
    }

    /// Return the visible y range as `(min, max)`.
    pub fn y_range(self) -> (f64, f64) {
        (self.y_min, self.y_max)
    }

    /// Return the camera center as `[x, y]`.
    pub fn center(self) -> [f64; 2] {
        [self.center_x, self.center_y]
    }

    /// Return the camera half extents as `[x, y]`.
    pub fn half_extents(self) -> [f64; 2] {
        [self.half_width, self.half_height]
    }

    /// Return the viewport size as `[width, height]`.
    pub fn viewport_size(self) -> [f64; 2] {
        [self.viewport_width, self.viewport_height]
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct PlotRenderUpdate {
    pub hover_pick: Option<HoverPickEvent>,
    pub drag_event: Option<DragEvent>,
    pub clear_cursor_position: bool,
    pub cursor_position_ui: Option<CursorPositionUiPayload>,
    pub x_ticks: Option<Vec<PositionedTick>>,
    pub y_ticks: Option<Vec<PositionedTick>>,
    /// Internal: Camera and bounds for coordinate conversion (only used internally, not part of public API)
    pub(crate) camera_bounds: Option<Box<(Camera, Rectangle)>>,
    pub(crate) view_change: Option<PlotViewChange>,
}

impl PlotRenderUpdate {
    /// Get the public view bounds snapshot for this render update.
    pub fn view_bounds(&self) -> Option<PlotViewBounds> {
        self.camera_bounds
            .as_deref()
            .map(|(camera, bounds)| PlotViewBounds::from_camera_bounds(camera, bounds))
    }

    /// Get the public view change snapshot for this render update.
    pub fn view_change(&self) -> Option<PlotViewChange> {
        self.view_change
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::DVec2;

    fn render_update_with_view_change(view_change: Option<PlotViewChange>) -> PlotRenderUpdate {
        let camera = Camera {
            position: DVec2::new(10.0, 20.0),
            half_extents: DVec2::new(3.0, 4.0),
            render_offset: DVec2::ZERO,
        };
        let bounds = Rectangle {
            x: 1.0,
            y: 2.0,
            width: 640.0,
            height: 480.0,
        };

        PlotRenderUpdate {
            hover_pick: None,
            drag_event: None,
            clear_cursor_position: false,
            cursor_position_ui: None,
            x_ticks: None,
            y_ticks: None,
            camera_bounds: Some(Box::new((camera, bounds))),
            view_change,
        }
    }

    #[test]
    fn plot_ui_message_exposes_view_bounds() {
        let message = PlotUiMessage::RenderUpdate(render_update_with_view_change(None));
        let bounds = message.get_view_bounds().unwrap();

        assert_eq!(bounds.x_range(), (7.0, 13.0));
        assert_eq!(bounds.y_range(), (16.0, 24.0));
        assert_eq!(bounds.center(), [10.0, 20.0]);
        assert_eq!(bounds.half_extents(), [3.0, 4.0]);
        assert_eq!(bounds.viewport_size(), [640.0, 480.0]);
    }

    #[test]
    fn non_render_messages_have_no_view_bounds() {
        assert_eq!(PlotUiMessage::ToggleLegend.get_view_bounds(), None);
    }

    #[test]
    fn plot_ui_message_exposes_view_change() {
        let bounds = render_update_with_view_change(None).view_bounds().unwrap();
        let view_change = PlotViewChange {
            bounds,
            x_zoomed: true,
            y_zoomed: false,
            panned: true,
            resized: false,
            x_axis_link_offset_changed: true,
        };
        let message =
            PlotUiMessage::RenderUpdate(render_update_with_view_change(Some(view_change)));

        assert_eq!(message.get_view_change(), Some(view_change));
        assert!(
            message
                .get_view_change()
                .unwrap()
                .x_axis_link_offset_changed
        );
    }

    #[test]
    fn non_render_messages_have_no_view_change() {
        assert_eq!(PlotUiMessage::ToggleLegend.get_view_change(), None);
    }
}

/// Drag interaction event in data/world coordinates.
#[derive(Debug, Clone, Copy)]
pub enum DragEvent {
    /// A drag gesture started inside the plot.
    Start {
        /// Current cursor world/data coordinate.
        world: [f64; 2],
    },
    /// Cursor moved while drag is active.
    Update {
        /// Current cursor world/data coordinate.
        world: [f64; 2],
    },
    /// Active drag gesture ended.
    End {
        /// Current cursor world/data coordinate.
        world: [f64; 2],
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Identifier for a point in a series.
pub struct PointId {
    /// ID of the series
    pub series_id: ShapeId,
    /// Index within the series [0..len)
    pub point_index: usize,
}

/// The hover or pick event.
#[derive(Debug, Clone, Copy)]
pub enum HoverPickEvent {
    /// Hover a point.
    Hover(PointId),
    /// Clear all hovered points.
    ClearHover,
    /// Pick a point.
    Pick(PointId),
    /// Clear all picked points.
    ClearPick,
}
