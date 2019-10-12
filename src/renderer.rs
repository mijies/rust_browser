use crate::layout::Dimensions;
use crate::painter::{DisplayCommand, DisplayList};

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

pub fn render(list: &DisplayList, viewport: &Dimensions) {
    let (doc, page1, layer1) = PdfDocument::new(
        "printpdf title",
        Mm(viewport.content.width),
        Mm(viewport.content.height),
        "Initial layer name"
    );
    let current_layer = doc.get_page(page1).get_layer(layer1);

    for display_command in list {
        render_points_by_display_command(&doc, &current_layer, &display_command, viewport);
    }
    doc.save(&mut BufWriter::new(File::create("pritpdf.pdf").unwrap())).unwrap();
}

fn render_points_by_display_command(
    doc: &types::pdf_document::PdfDocumentReference,
    layer: &types::pdf_layer::PdfLayerReference,
    display_command: &DisplayCommand,
    viewport: &Dimensions
) {
    match display_command {
        &DisplayCommand::SolidColor(ref color, rect) => {
            let y_top = Mm(360.0 - (rect.y + rect.height));
            let y_bottom = Mm(360.0 - rect.y);
            // x and y positions from the bottom left corner clockwise
            let points = vec![
                (Point::new(Mm(rect.x), y_bottom), false),
                (Point::new(Mm(rect.x), y_top), false),
                (Point::new(Mm(rect.x + rect.width), y_top), false),
                (Point::new(Mm(rect.x + rect.width), y_bottom), false),
            ];
            layer.set_fill_color(Color::Rgb(
                Rgb::new(
                    color.r as f64 / 255.0,
                    color.g as f64 / 255.0,
                    color.b as f64 / 255.0,
                    None
            )));
            layer.add_shape(Line {
                points: points,
                is_closed: true,
                has_fill: true,
                has_stroke: true,
                is_clipping_path: false,
            });
        }
        &DisplayCommand::Text(ref content, rect) => {
            let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
            
            layer.set_fill_color(Color::Rgb(
                Rgb::new(0.0, 0.0, 0.0, None) // enum Color from printpdf
            ));
            layer.use_text(
                content.as_str(),
                16 * 3, // font size
                Mm(rect.x),
                Mm(360.0 - rect.y - rect.height),
                &font // font: &IndirectFontRef
            );
        }
    }
}