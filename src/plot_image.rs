use std::sync::Arc;

use crate::{
    Color,
    series::{SeriesError, ShapeId},
    transform::{PositionTransform, Transform},
};

/// RGBA image drawn inside the plot in data coordinates.
///
/// `PlotImage` is intended for dense raster data such as spectrograms. The
/// image is positioned by a center point and size in plot coordinates, matching
/// the core `egui_plot::PlotImage` model, and can also be constructed from
/// explicit data-space bounds.
#[derive(Debug, Clone)]
pub struct PlotImage {
    /// Unique identifier for this image.
    pub id: ShapeId,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image bytes in unpremultiplied RGBA8 order.
    pub rgba: Arc<[u8]>,
    /// Center of the image in data coordinates before transforms.
    pub center: [f64; 2],
    /// Size of the image in data units before transforms.
    pub size: [f64; 2],
    /// Normalized UV rectangle, as `[min, max]`.
    pub uv: [[f32; 2]; 2],
    /// How this image interprets or converts x/y values before drawing.
    pub transform: PositionTransform,
    /// Optional label for legends.
    pub label: Option<String>,
    /// Color multiplied with the image. Defaults to white.
    pub tint: Color,
    /// Solid color drawn behind the image. Useful for transparent image data.
    pub bg_fill: Color,
}

impl PlotImage {
    /// Create an image from raw RGBA bytes, a center point, and size in data units.
    pub fn from_rgba(
        width: u32,
        height: u32,
        rgba: impl Into<Arc<[u8]>>,
        center: [f64; 2],
        size: [f64; 2],
    ) -> Self {
        Self {
            id: ShapeId::new(),
            width,
            height,
            rgba: rgba.into(),
            center,
            size,
            uv: [[0.0, 0.0], [1.0, 1.0]],
            transform: PositionTransform::identity(),
            label: None,
            tint: Color::WHITE,
            bg_fill: Color::TRANSPARENT,
        }
    }

    /// Create an image from raw RGBA bytes and data-space bounds.
    pub fn from_rgba_bounds(
        width: u32,
        height: u32,
        rgba: impl Into<Arc<[u8]>>,
        min: [f64; 2],
        max: [f64; 2],
    ) -> Self {
        let center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        let size = [max[0] - min[0], max[1] - min[1]];
        Self::from_rgba(width, height, rgba, center, size)
    }

    /// Set image bounds in data coordinates.
    pub fn with_bounds(mut self, min: [f64; 2], max: [f64; 2]) -> Self {
        self.center = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5];
        self.size = [max[0] - min[0], max[1] - min[1]];
        self
    }

    /// Set image center and size in data units.
    pub fn with_center_size(mut self, center: [f64; 2], size: [f64; 2]) -> Self {
        self.center = center;
        self.size = size;
        self
    }

    /// Set a normalized UV rectangle, as `[min, max]`.
    ///
    /// The default is `[[0.0, 0.0], [1.0, 1.0]]`, where `(0, 0)` is the
    /// top-left of the source image.
    pub fn with_uv(mut self, min: [f32; 2], max: [f32; 2]) -> Self {
        self.uv = [min, max];
        self
    }

    /// Set a label for this image (shown in legend when non-empty).
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        let label = label.into();
        if !label.is_empty() {
            self.label = Some(label);
        }
        self
    }

    /// Set image tint. White preserves the original image colors.
    pub fn with_tint(mut self, tint: impl Into<Color>) -> Self {
        self.tint = tint.into();
        self
    }

    /// Set a background color behind the image.
    pub fn with_bg_fill(mut self, bg_fill: impl Into<Color>) -> Self {
        self.bg_fill = bg_fill.into();
        self
    }

    /// Set how this image interprets or converts x/y values before drawing.
    pub fn with_transform(mut self, transform: PositionTransform) -> Self {
        self.transform = transform;
        self
    }

    /// Set how this image interprets or converts x values before drawing.
    pub fn with_transform_x(mut self, transform: Transform) -> Self {
        self.transform.x = Some(transform);
        self
    }

    /// Set how this image interprets or converts y values before drawing.
    pub fn with_transform_y(mut self, transform: Transform) -> Self {
        self.transform.y = Some(transform);
        self
    }

    /// Interpret image center and size as normalized plot coordinates.
    pub fn with_axes_transform(mut self) -> Self {
        self.transform = PositionTransform::axes();
        self
    }

    /// Return data-space bounds as `(min, max)` before transforms.
    pub fn bounds(&self) -> ([f64; 2], [f64; 2]) {
        let half = [self.size[0] * 0.5, self.size[1] * 0.5];
        (
            [self.center[0] - half[0], self.center[1] - half[1]],
            [self.center[0] + half[0], self.center[1] + half[1]],
        )
    }

    pub(super) fn validate(&self) -> Result<(), SeriesError> {
        if self.width == 0 || self.height == 0 {
            return Err(SeriesError::InvalidImageDimensions);
        }
        let expected = self.width as usize * self.height as usize * 4;
        if self.rgba.len() != expected {
            return Err(SeriesError::InvalidImageDataLength {
                expected,
                actual: self.rgba.len(),
            });
        }
        if !self.center.iter().all(|value| value.is_finite())
            || !self
                .size
                .iter()
                .all(|value| value.is_finite() && *value > 0.0)
            || !self.uv.iter().flatten().all(|value| value.is_finite())
        {
            return Err(SeriesError::InvalidImageGeometry);
        }
        Ok(())
    }
}
