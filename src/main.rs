// extern crate clap;
// use clap::{App, Arg};

extern crate rust_browser;
use rust_browser::renderer;

// const VERSION_STR: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    // let app = App::new("rust_browser")
    //     .version(VERSION_STR)
    //     .author("mijies")
    //     .about("Web browser implementation in Rust")
    //     .arg(Arg::with_name("FILE")
    //         .help("Input file")
    //         .index(1)
    //     );
    // let _app_matches = app.get_matches();
    renderer::f();
}