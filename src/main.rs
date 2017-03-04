#[macro_use]
mod macros;
mod constants;

mod rustmap;
mod config;
mod gui;
mod position;
mod maparea;
mod tileset;

#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate cairo;

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

    let mut config_file = File::open("rustmap.toml").unwrap();
    let mut config_string = String::new();
    if let Err(err) = config_file.read_to_string(&mut config_string) {
        println!("{}", err);
        return;
    }
    let config: Config = toml::from_str(&mut config_string).unwrap();
    let builder = Builder::new_from_file("builder.ui");

    let gui = Gui::new(config, builder);
    gui.run();

}
