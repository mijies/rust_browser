
use rust_browser::html;
use rust_browser::css;
use rust_browser::style;
use rust_browser::layout;

use clap::{App, Arg};

use std::default::Default;
use std::fs::OpenOptions;
use std::io::prelude::*;

const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let app = App::new("rust_browser")
        .version(VERSION_STR)
        .author("mijies")
        .about("Web browser implementation in Rust")
        .arg(Arg::with_name("FILE")
            .help("Input file")
            .index(1)
        );
    let _app_matches = app.get_matches();
    
    println!("HTML:");
    let mut html_source = "".to_string();
    OpenOptions::new()
        .read(true)
        .open("./example/test.html")
        .unwrap()
        .read_to_string(&mut html_source)
        .ok()
        .expect("cannot read file");
    let html_tree = html::parse(html_source);
    println!("{}", html_tree);

    println!("CSS:");
    let mut css_source = "".to_string();
    OpenOptions::new()
        .read(true)
        .open("example/test.css")
        .unwrap()
        .read_to_string(&mut css_source)
        .ok()
        .expect("cannot read file");
    let stylesheet = css::parse(css_source);
    css::show_css(&stylesheet);

    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;

    let style_tree = style::style_tree(&html_tree, &stylesheet);
    let layout_tree = layout::layout_tree(&style_tree, viewport);
    println!("{}", layout_tree);
}