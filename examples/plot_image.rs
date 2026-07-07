//! Example showing how to render an in-memory raster via [`PlotImage`].
use iced::Element;
use iced_plot::{
    Color, HLine, LineStyle, PlotImage, PlotUiMessage, PlotWidget, PlotWidgetBuilder, Series,
};

fn main() -> iced::Result {
    iced::application(new, update, view)
        .font(include_bytes!("fonts/FiraCodeNerdFont-Regular.ttf"))
        .default_font(iced::Font::with_name("FiraCode Nerd Font"))
        .run()
}

fn update(widget: &mut PlotWidget, message: PlotUiMessage) {
    widget.update(message);
}

fn view(widget: &PlotWidget) -> Element<'_, PlotUiMessage> {
    widget.view()
}

fn new() -> PlotWidget {
    const IMAGE_WIDTH: u32 = 240;
    const IMAGE_HEIGHT: u32 = 140;
    let image_pixels = build_raster(IMAGE_WIDTH, IMAGE_HEIGHT);

    let image = PlotImage::from_rgba_bounds(
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        image_pixels,
        [0.0, -4.0],
        [12.0, 4.0],
    )
    .with_label("in-memory RGBA image");

    let guide_points = (0..241)
        .map(|i| {
            let t = i as f64 / 240.0;
            let x = t * 12.0;
            let y = (t * std::f64::consts::PI * 1.75).sin() * 1.6;
            [x, y]
        })
        .collect::<Vec<_>>();

    let guide_line = Series::line_only(guide_points, LineStyle::solid().with_pixel_width(2.5))
        .with_label("guide curve")
        .with_color(Color::from_rgb(0.98, 0.95, 0.18));

    let midline = HLine::new(0.0)
        .with_label("zero-reference")
        .with_color(Color::from_rgba(1.0, 0.34, 0.34, 0.75))
        .with_width(1.4);

    PlotWidgetBuilder::new()
        .add_image(image)
        .add_series(guide_line)
        .add_hline(midline)
        .with_x_label("time (s)")
        .with_y_label("frequency (a.u.)")
        .with_tick_label_size(12.0)
        .with_axis_label_size(16.0)
        .with_cursor_overlay(true)
        .with_cursor_provider(|x, y| format!("x: {x:.2}, y: {y:.2}"))
        .with_crosshairs(true)
        .build()
        .unwrap()
}

fn build_raster(width: u32, height: u32) -> Vec<u8> {
    let mut pixels = Vec::with_capacity((width * height * 4) as usize);

    for y in 0..height {
        for x in 0..width {
            let xu = x as f64 / (width as f64 - 1.0);
            let yu = y as f64 / (height as f64 - 1.0);

            let stripe = (((x / 12 + y / 12) % 2) as f64) * 35.0;
            let wave =
                (xu * 12.0 * std::f64::consts::PI).sin() * (yu * 8.0 * std::f64::consts::PI).cos();

            let mut red = (xu * 220.0 + 25.0 + wave * 12.0 + stripe).clamp(0.0, 255.0) as u8;
            let mut green = (yu * 220.0 + 20.0 - wave * 18.0 + stripe).clamp(0.0, 255.0) as u8;
            let mut blue = ((128.0 + 110.0 * wave + stripe).clamp(0.0, 255.0)) as u8;
            let alpha = 255u8;

            if x < 20 && y < 20 {
                red = 255;
                green = 0;
                blue = 0;
            } else if x > width.saturating_sub(20) && y > height.saturating_sub(20) {
                red = 0;
                green = 255;
                blue = 255;
            }

            pixels.extend_from_slice(&[red, green, blue, alpha]);
        }
    }

    pixels
}
