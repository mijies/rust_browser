use crate::style::{Display, StyledNode};
use crate::css::{Unit, Value};
use crate::css::Value::{Keyword, Length};
use crate::dom::NodeType;
use std::default::Default;
use std::fmt;

pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Dimensions {
    pub content: Rect, // relative to the document origin
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x : f64,
    pub y : f64,
    pub width : f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeSizes {
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

// Transform a style tree into a layout tree
pub fn layout_tree<'a>(
    node: &'a StyledNode<'a>, 
    mut containing_block: Dimensions // https://www.w3.org/TR/CSS2/visudet.html#containing-block-details
) -> LayoutBox<'a> {
    containing_block.content.height = 0.0;
    let mut root_box = make_layout_tree(node);
    root_box.layout(containing_block);
    root_box
}

// Make a layout tree but no layout calcualtions performed
fn make_layout_tree<'a>(node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match node.display() {
        Display::Block => BoxType::BlockNode(node),
        Display::Inline => BoxType::InlineNode(node),
        Display::None => panic!("Root node has display: none"),
    });

    for child in &node.children {
        match child.display() {
            Display::Block => root.children.push(make_layout_tree(child)),
            Display::Inline => root.get_inline_container()
                .children.push(make_layout_tree(child)),
            Display::None => {},
        }
    }
    root
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType<'a>) -> LayoutBox<'a> {
        LayoutBox {
            dimensions: Default::default(),
            box_type: box_type,
            children: Vec::new(),
        }
    }

    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => self.layout_inline(containing_block),
            BoxType::AnonymousBlock => for child in &mut self.children {
                child.layout(containing_block);
                self.dimensions.content.width = child.dimensions.margin_box().width;
                self.dimensions.content.height += child.dimensions.margin_box().height;
            },
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block); // position in its container
        self.layout_block_children();  // dependent on its parent width
        self.calculate_block_height(); // dependent on its children height
    }

    // TODO: checkout if not violate the regurations
    // https://www.w3.org/TR/CSS2/visudet.html#blockwidth
    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let auto = Keyword("auto".to_string()); // initial vaule
        let zero = Length(0.0, Unit::Px);       // initial vaule for margin border padding

        let mut width = style.value("width").unwrap_or(auto.clone());
        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);
        let mut border_left = style.lookup("border-left-width", "border-width", &zero);
        let mut border_right = style.lookup("border-right-width", "border-width", &zero);
        let mut padding_left = style.lookup("padding-left", "padding", &zero);
        let mut padding_right = style.lookup("padding-right", "padding", &zero);

        let total: f64 = [
            &margin_right,
            &border_right,
            &padding_right,
            &padding_left,
            &border_left,
            &margin_left,
            &width
        ].iter().map(|v| v.to_px()).sum();  // 0.0 if not Value::Length

        let mut underflow = containing_block.content.width - total;
        if underflow < 0.0 {
            // 0.0 if auto
            width = Length(width.to_px(), Unit::Px);
            margin_left = Length(margin_left.to_px(), Unit::Px);
            margin_right = Length(margin_right.to_px(), Unit::Px);
            border_left = Length(border_left.to_px(), Unit::Px);
            border_right = Length(border_right.to_px(), Unit::Px);
            padding_left = Length(padding_left.to_px(), Unit::Px);
            padding_right = Length(padding_right.to_px(), Unit::Px);

            // reduce the length from the rightmost
            underflow = self.consume_underflow(&mut underflow, &mut margin_right);
            underflow = self.consume_underflow(&mut underflow, &mut border_right);
            underflow = self.consume_underflow(&mut underflow, &mut padding_right);
            underflow = self.consume_underflow(&mut underflow, &mut padding_left);
            underflow = self.consume_underflow(&mut underflow, &mut border_left);
            underflow = self.consume_underflow(&mut underflow, &mut margin_left);
            self.consume_underflow(&mut underflow, &mut width);
        } else {
            if width == auto {
                // only width consumes the length of underflow
                width = Length(underflow, Unit::Px);

                // 0.0 if auto
                margin_left = Length(margin_left.to_px(), Unit::Px);
                margin_right = Length(margin_right.to_px(), Unit::Px);
                border_left = Length(border_left.to_px(), Unit::Px);
                border_right = Length(border_right.to_px(), Unit::Px);
                padding_left = Length(padding_left.to_px(), Unit::Px);
                padding_right = Length(padding_right.to_px(), Unit::Px);        
            } else {
                // TODO: handle auto combinations
                margin_right = Length(margin_right.to_px() + underflow, Unit::Px)
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();
        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();
        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();
    }

    fn consume_underflow(&mut self, underflow: &mut f64, value: &mut Value) -> f64 {
        let flow = value.to_px() + *underflow;
        if flow > 0.0 {
            *value = Length(flow, Unit::Px);
            0.0
        } else {
            *value = Length(0.0, Unit::Px);
            flow
        }
    }

    // TODO: checkout if not violate the regurations
    // https://www.w3.org/TR/CSS2/visudet.html#normal-block
    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let zero = Length(0.0, Unit::Px); // initial vaule for margin border padding
        let d = &mut self.dimensions;

        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();
        d.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        d.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();
        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y = containing_block.content.height // add up the previous boxes in the container
            + containing_block.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(*d);
            d.content.height += child.dimensions.margin_box().height; // add up
        }
    }

    fn calculate_block_height(&mut self) {
        if let Some(Length(h, Unit::Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h; // override the height by children if explicitly set
        }
    }

    fn layout_inline(&mut self, containing_block: Dimensions) {
        self.calculate_inline_position(containing_block); // position in its container
        self.layout_inline_children();
        
        // if the node is text, the width and height of the text become of the node
        match self.get_style_node().node.data {
            NodeType::Element(_) => {}
            NodeType::Text(ref body) => {
                // TODO: fix the hardcodeds
                self.dimensions.content.width = body.len() as f64 * 8.0;
                self.dimensions.content.height = 16.0;
            }
        }
    }

    // TODO: checkout if not violate the regurations
    // https://www.w3.org/TR/CSS2/visudet.html#inline-width
    fn calculate_inline_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;
        let zero = Length(0.0, Unit::Px); // initial vaule for margin border padding

        d.margin.left = style.lookup("margin-left", "margin", &zero).to_px();
        d.margin.right = style.lookup("margin-right", "margin", &zero).to_px();
        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        // Inline has no border and padding left/right?
        d.border.top = style.lookup("border-top-width", "border-width", &zero).to_px();
        d.border.bottom = style.lookup("border-bottom-width", "border-width", &zero).to_px();
        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;
        d.content.y = containing_block.content.height // add up the previous boxes in the container
            + containing_block.content.y + d.margin.top + d.border.top + d.padding.top;
    }

    fn layout_inline_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(*d);
            d.content.width = child.dimensions.margin_box().width;
            d.content.height += child.dimensions.margin_box().height; // add up
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block box has no style node"),
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) => { // requires AnonymousBlock to host an inline box
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::AnonymousBlock,
                        ..
                    }) => {}, // make use of the previous AnonymousBlock
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }

}

impl Dimensions {
    fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(self.margin)
    }

    fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(self.border)
    }

    fn padding_box(&self) -> Rect {
        self.content.expanded_by(self.padding)
    }
}

impl Rect {
    fn expanded_by(&self, edge: EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top * edge.bottom,
        }
    }
}

impl<'a> fmt::Display for LayoutBox<'a> { // type Result = Result<(), Error>;
    // TODO: implement more later
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.dimensions)?;
        for child in &self.children {
            write!(f, "{}", child)?;
        }
        Ok(())
    }
}