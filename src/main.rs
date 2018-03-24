#[macro_use]
mod macros;
mod constants;

mod rustmap;
mod config;
mod gui;
mod history;
mod maparea;
mod position;
mod palette;
mod tileset;

#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate cairo;
extern crate gdk;
extern crate gdk_pixbuf;
extern crate gtk;

use std::io::prelude::*;
use std::fs::File;

use gtk::Builder;

use config::Config;
use gui::Gui;

fn main () {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let config_filename = "rustmap.toml";
    let mut config_file = File::open(config_filename).expect(&format!("Invalid toml file: {}", config_filename));
    let mut config_string = String::new();
    let config: Option<Config> = config_file.read_to_string(&mut config_string)
        .ok().and_then(|_| toml::from_str(&mut config_string).ok());
    let builder = Builder::new_from_string(include_str!("builder.ui"));

    let mut gui = Gui::new(config, builder);
    gui.run();
}
