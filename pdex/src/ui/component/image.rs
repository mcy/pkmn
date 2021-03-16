//! Components for displaying images rendered as text.

use std::fmt::Debug;

use pkmn::api::Blob;

use tui::layout::Alignment;
use tui::layout::Rect;
use tui::style::Color;
use tui::style::Modifier;
use tui::style::Style;
use tui::text::Span;
use tui::text::Spans;
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::widgets::Widget;

use crate::ui::component::Component;
use crate::ui::component::RenderArgs;

#[derive(Clone, Debug)]
pub struct Png {
  blob: Blob,
  cache: Option<(Rect, Text<'static>)>,
}

impl Png {
  pub fn new(blob: Blob) -> Self {
    Self { blob, cache: None }
  }
}

impl Component for Png {
  fn render(&mut self, args: &mut RenderArgs) {
    if args.rect.height == 0 || args.rect.width == 0 {
      return;
    }

    let text = match &self.cache {
      Some((rect, chars))
        if rect.height == args.rect.height && rect.width == args.rect.width =>
      {
        chars
      }
      _ => match args.dex.load_png(&self.blob) {
        None => return,
        Some(image) => {
          // NOTE: Wider rectangles have a smaller aspect ratio, while taller
          // rectangles have a greater one.
          let rect_aspect = args.rect.height as f64 / args.rect.width as f64;
          let image_aspect = image.height() as f64 / image.width() as f64;

          // If the draw rectangle is wider or shorter than the image, we scale
          // according to the height ratio; otherwise, we use the width.
          let (width, height) = if rect_aspect * args.style_sheet.font_height
            < image_aspect
          {
            let scale_factor = args.rect.height as f64 / image.height() as f64;

            let width = (image.width() as f64
              * scale_factor
              * args.style_sheet.font_height) as u32;
            let height = (image.height() as f64 * scale_factor) as u32;

            (width, height)
          } else {
            let scale_factor = args.rect.width as f64 / image.width() as f64;

            let width = (image.width() as f64 * scale_factor) as u32;
            let height = (image.height() as f64 * scale_factor
              / args.style_sheet.font_height) as u32;

            (width, height)
          };

          // Recolor the transparent image parts to be black instead of white, so
          // as to improve resizing.
          let mut image = (&*image).clone();
          for image::Rgba([r, g, b, a]) in image.pixels_mut() {
            if *a == 0 {
              *r = 0;
              *g = 0;
              *b = 0;
            }
          }

          // We resize twice; once with nearest-neighbor and once with triangle
          // interpolation. The NN version is only used for alpha masking.
          let mask = image::imageops::resize(
            &image,
            width,
            height,
            image::imageops::FilterType::Nearest,
          );
          let mut resized = image::imageops::resize(
            &image,
            width,
            height,
            image::imageops::FilterType::Triangle,
          );

          for (image::Rgba([_, _, _, a]), image::Rgba([_, _, _, out])) in
            mask.pixels().zip(resized.pixels_mut())
          {
            *out = *a;
          }

          // Now, we rasterize. For now we just do a very dumb thing.
          let mut text = Text::default();
          for row in resized.rows() {
            let mut spans = Vec::new();
            for &image::Rgba([r, g, b, a]) in row {
              let s = if a != 0 { "@" } else { " " };
              spans.push(Span::styled(
                s,
                Style::default()
                  .fg(Color::Rgb(r, g, b))
                  .add_modifier(Modifier::BOLD),
              ));
            }
            text.lines.push(Spans::from(spans));
          }
          self.cache = Some((args.rect, text));
          &self.cache.as_ref().unwrap().1
        }
      },
    };

    let dy = args.rect.height.saturating_sub(text.lines.len() as u16) / 2;
    let rect = Rect::new(
      args.rect.x,
      args.rect.y + dy,
      args.rect.width,
      text.lines.len() as u16,
    );
    Paragraph::new(text.clone())
      .alignment(Alignment::Center)
      .render(rect, args.output);
  }
}
