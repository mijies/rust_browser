use crate::css::{Color};
use crate::dom::NodeType;
use crate::layout::{BoxType, LayoutBox, Rect};


pub struct Canvas {
    pub pixels: Vec<Color>,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug)]
pub enum DisplayCommand {
    SolidColor(Color, Rect),
    Text(String, Rect),
}

pub type DisplayList = Vec<DisplayCommand>;

impl Canvas {
    fn new(width: usize, height: usize) -> Canvas {
        let white = Color {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        };
        Canvas {
            pixels: vec![white; width * height],
            width: width,
            height: height,
        }
    }

    fn paint_pixels_by_display_command(&mut self, display_command: &DisplayCommand) {
        match *display_command {
            DisplayCommand::SolidColor(color, rect) => {
                // clip out the canvas rectangle boundaries.
                let x_left = rect.x.max(0.0).min(self.width as f64) as usize;
                let y_top = rect.y.max(0.0).min(self.height as f64) as usize;
                let x_right = (rect.x + rect.width).max(0.0).min(self.width as f64) as usize;
                let y_bottom = (rect.y + rect.height).max(0.0).min(self.height as f64) as usize;

                for y in y_top..y_bottom {
                    for x in x_left..x_right {
                        self.pixels[y * self.width + x] = color;
                    }
                }
            }
            _ => {}
        }
    }
}

// make a pixel array from the layout tree
pub fn paint(layout_root: &LayoutBox, boundary: Rect) -> Canvas {
    let mut display_command_list = Vec::new();
    render_layout_box_tree(&mut display_command_list, layout_root);

    let mut canvas = Canvas::new(boundary.width as usize, boundary.height as usize);
    for display_command in &display_command_list {
        canvas.paint_pixels_by_display_command(display_command);
    }
    canvas
}

pub fn render_layout_box_tree(list: &mut DisplayList, layout_box: &LayoutBox) {
    render_text(list, layout_box);
    render_background(list, layout_box);
    render_border(list, layout_box);
    for child in &layout_box.children {
        render_layout_box_tree(list, child);
    }
}

fn render_text(list: &mut DisplayList, layout_box: &LayoutBox) {
    match layout_box.box_type {
        BoxType::BlockNode(style_node) | BoxType::InlineNode(style_node)
            => match style_node.node.data {
                NodeType::Text(ref content) => list.push(
                    DisplayCommand::Text(
                        content.clone(), 
                        layout_box.dimensions.border_box(),
                )),
                NodeType::Element(_) => (),
            }
        _ => (),
    }
}

fn render_background(list: &mut DisplayList, layout_box: &LayoutBox) {
    get_color(layout_box, "background").map(|color| 
        list.push(DisplayCommand::SolidColor(
            color,
            layout_box.dimensions.border_box(),
        ))
    );
}

fn render_border(list: &mut DisplayList, layout_box: &LayoutBox) {
    let color = match get_color(layout_box, "border-color") {
        Some(color) => color,
        _ => return,
    };

    let d = layout_box.dimensions;
    let border_box = d.border_box();

    // left border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: d.border.left,
            height: border_box.height,
        }
    ));

    // right border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x + border_box.width - d.border.right,
            y: border_box.y,
            width: d.border.right,
            height: border_box.height,
        }
    ));

    // top border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y,
            width: border_box.width,
            height: d.border.top,
        }
    ));

    // bottom border
    list.push(DisplayCommand::SolidColor(
        color,
        Rect {
            x: border_box.x,
            y: border_box.y + border_box.height - d.border.bottom,
            width: border_box.width,
            height: d.border.bottom,
        }
    ));
}

fn get_color(layout_box: &LayoutBox, name: &str) -> Option<Color> {
    match layout_box.box_type {
        BoxType::BlockNode(style) | BoxType::InlineNode(style) 
            => style.get_color(name),
        BoxType::AnonymousBlock => None,
    }
}