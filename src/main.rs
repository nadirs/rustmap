#[macro_use]
mod macros;
mod constants;

mod rustmap;
mod position;
mod maparea;
mod tileset;

extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate cairo;

use std::io::prelude::*;
use std::fs::File;

use gtk::prelude::*;
use gtk::{Builder, DrawingArea, Label, Window};
use gdk::Gravity;
use gdk_pixbuf::Pixbuf;

use tileset::Tileset;
use maparea::Maparea;
use position::get_event_pos;

fn get_bytes_from_filepath(path: &str) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(&path).map(|mut f| {
        let _ = f.read_to_end(&mut bytes);
        bytes
    }).ok()
}

fn main () {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let mut argv = std::env::args();
    let map_path = argv.nth(1).unwrap_or("../../../maps/viridiancity.blk".to_string());
    let tileset_path = argv.nth(1).unwrap_or("../../../gfx/tilesets/overworld.t2.png".to_string());
    let blockset_path = argv.nth(2).unwrap_or("../../../gfx/blocksets/overworld.bst".to_string());

    let mut mapset: Vec<u8> = Vec::new();
    let mut mapset_file = File::open(&map_path).unwrap();
    if let Err(err) = mapset_file.read_to_end(&mut mapset) {
        println!("{}", err);
    }

    let builder = Builder::new_from_file("builder.ui");
    let window: Window = builder.get_object("window").unwrap();
    let lbl_coords: Label = builder.get_object("lblCoords").unwrap();
    let tileset_widget: DrawingArea = builder.get_object("tileset").unwrap();
    let maparea_widget: DrawingArea = builder.get_object("image").unwrap();

    let blockset: Vec<u8> = get_bytes_from_filepath(&blockset_path).unwrap();
    let tileset_pix = Pixbuf::new_from_file(&tileset_path).unwrap();

    // Rc<RefCell<Tileset>>
    // Usage:
    //     tileset.borrow_mut().some_tileset_method()
    let tileset = Tileset::from_data(&blockset, &tileset_widget, &tileset_pix);
    tileset.borrow_mut().select_tile_at(0);

    let maparea = Maparea::new(20, 18, mapset, tileset);
    maparea::connect_events(maparea, &maparea_widget);

    maparea_widget.connect_button_press_event(move |_, ev| {
        let pos = get_event_pos(ev.get_position());
        lbl_coords.set_label(&format!("{:?}", pos));

        Inhibit::default()
    });

    // UI initialization.
    window.set_gravity(Gravity::Center);
    window.show_all();

    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    // Run the main loop.
    gtk::main();
}
