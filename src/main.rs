mod rustmap;

extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate cairo;

use std::io::prelude::*;
use std::fs::File;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{Builder, Button, ButtonsType, DialogFlags, EventBox, DrawingArea, Image, Label, Layout, MessageType, MessageDialog, Window, WindowType};
use gdk::prelude::*;
use gdk::{Gravity, EventTouch, EventType, EventMask};
use gdk_pixbuf::{Pixbuf, InterpType};
use cairo::Context;
//
// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

const TILE_SIZE: usize = 8;
const TILES_IN_ROW: usize = 4;
const TILES_IN_COL: usize = 4;
const BLOCK_SIZE: usize = TILE_SIZE * TILES_IN_ROW;
const TILES_IN_BLOCK: usize = TILES_IN_ROW * TILES_IN_COL;

fn main () {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let mut argv = std::env::args();
    //let tileset_path = argv.nth(1).unwrap_or("../../tilesets/Tset00_GFX.png".to_string());
    let tileset_path = argv.nth(1).unwrap_or("../../../gfx/tilesets/overworld.t2.png".to_string());
    let blockset_path = argv.nth(2).unwrap_or("../../../gfx/blocksets/overworld.bst".to_string());
    println!("{}", tileset_path);
    println!("{}", blockset_path);

    let mut blockset: Vec<u8> = Vec::new();
    let mut blockset_file = File::open(&blockset_path).unwrap();
    blockset_file.read_to_end(&mut blockset);

    let builder = Builder::new_from_file("builder.ui");

    let window: Window = builder.get_object("window").unwrap();

    let image: DrawingArea = builder.get_object("image").unwrap();
    let tileset: DrawingArea = builder.get_object("tileset").unwrap();
    let bg = gdk::RGBA { red: 1., green: 0., blue: 0., alpha: 1. };


    let pix = Pixbuf::new_from_file(&tileset_path).unwrap();
    let mask = gdk_sys::GDK_POINTER_MOTION_MASK | gdk_sys::GDK_BUTTON_PRESS_MASK | gdk_sys::GDK_BUTTON1_MOTION_MASK;
    tileset.add_events(mask.bits() as i32);

    let lblCoords: Label = builder.get_object("lblCoords").unwrap();

    let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
    let context = cairo::Context::new(&surface);

    let mut x: u8 = 0;
    let mut selected: u8 = 0;
    let mut hovered: u8 = 0;
    let mut y: u8 = 0;
    let mut cell = Rc::new(RefCell::new((x, y, selected, hovered)));

    // UI initialization.
    window.set_gravity(Gravity::Center);
    window.show_all();

    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    tileset.connect_motion_notify_event(clone!(cell => move |el, ev| {
        let (ref mut x, ref mut y, ref mut selected, ref mut hovered) = *cell.borrow_mut();
        let pos = get_event_pos(ev.get_position());
        let (lx, ly) = pos;
        *x = lx;
        if lx != *hovered {
            el.queue_draw_area((*hovered as usize * BLOCK_SIZE) as i32, 0, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
            *hovered = *x;
            el.queue_draw_area((*hovered as usize * BLOCK_SIZE) as i32, 0, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
        }

        lblCoords.set_label(&format_event_pos(pos));
        Inhibit::default()
    }));

    image.connect_button_press_event(clone!(cell => move |el, ev| {
        Inhibit::default()
    }));

    tileset.connect_button_press_event(clone!(cell => move|el, ev| {
        let (ref mut x, ref mut y, ref mut selected, ref mut hovered) = *cell.borrow_mut();
        let pos = get_event_pos(ev.get_position());
        let (lx, ly) = pos;
        *x = lx;
        *selected = *x;
        *y = ly;
        println!("Clicked at {:?}", &format_event_pos(pos));
        el.queue_draw(); // replace with queue_draw_area (old block and new block)

        Inhibit::default()
    }));

    tileset.set_size_request(blockset.len() as i32 * 4, BLOCK_SIZE as i32);
    tileset.connect_draw(clone!(cell => move |el, context| {
        let tileset_width = pix.get_width() / 8 as i32;

        let (ref mut x, ref mut y, ref mut selected, ref mut hovered) = *cell.borrow_mut();

        el.override_background_color(gtk::StateFlags::all(), &gdk_sys::GdkRGBA { red: 255., green: 1., blue: 0., alpha: 1. });
        context.paint();

        for (i, b_) in blockset.iter().enumerate() {
            let b = *b_ as i32;
            let mut tile = pix.new_subpixbuf(8 * (b % tileset_width), 8 * ((b / tileset_width) as i32), 8, 8).scale_simple(TILE_SIZE as i32, TILE_SIZE as i32, InterpType::Nearest).unwrap();

            if (i / TILES_IN_BLOCK) as usize == *selected as usize {
                let mut pxs: Vec<u8> = Vec::new();
                unsafe {
                    for (i, b) in tile.get_pixels().iter().enumerate() {
                        let b1: u8 = match *b {
                            170 => match i % 3 {
                                0 => 0,
                                1 => 200,
                                2 => 0,
                                _ => panic!("foobar")
                            },
                            85 => match i % 3 {
                                0 => 0,
                                1 => 100,
                                2 => 0,
                                _ => panic!("fizzbuzz")
                            },
                            x => x
                        };
                        pxs.push(b1);
                    }
                };
                let ctile = Pixbuf::new_from_vec(pxs, tile.get_colorspace(), false, tile.get_bits_per_sample(), tile.get_width(), tile.get_height(), tile.get_rowstride());
                tile = ctile;
            }

            if (i / TILES_IN_BLOCK) as usize == *hovered as usize {
                let mut pxs: Vec<u8> = Vec::new();
                unsafe {
                    for (i, b) in tile.get_pixels().iter().enumerate() {
                        let b1: u8 = match *b {
                            170 => match i % 3 {
                                0 => 255,
                                1 => 0,
                                2 => 0,
                                _ => panic!("foobar")
                            },
                            85 => match i % 3 {
                                0 => 130,
                                1 => 0,
                                2 => 0,
                                _ => panic!("fizzbuzz")
                            },
                            x => x
                        };
                        pxs.push(b1);
                    }
                };
                let ctile = Pixbuf::new_from_vec(pxs, tile.get_colorspace(), false, tile.get_bits_per_sample(), tile.get_width(), tile.get_height(), tile.get_rowstride());
                tile = ctile;
            }

            context.set_source_pixbuf(&tile, (((i % 4) * 8) + (i / 16) * BLOCK_SIZE) as f64, ((((i / 4) % 4) as i32) * 8) as f64);
            context.paint();
        }
        context.save();
        Inhibit::default()
    }));

    /*
    tileset.connect_draw(move |el, cr| {
        //let drawable = el.get_window().unwrap();
        //let context = Context::create_from_window(&drawable);

        let tileset_width = pix.get_width() / 8 as i32;
        for (i, b_) in blockset.iter().enumerate() {
            let b = *b_ as i32;
            let tile = pix.new_subpixbuf(8 * (b % tileset_width), 8 * ((b / tileset_width) as i32), 8, 8).scale_simple(16, 16, InterpType::Nearest).unwrap();
            el.set_source_pixbuf(&pix, (((i % 4) * 16) + (i / 16) * 64) as f64, ((((i / 4) % 4) as i32) * 16) as f64);
            break;
        }
        Inhibit::default()
    });
    */

    // Run the main loop.
    gtk::main();
}

/**
 * on paint: read map data and draw tiles according to it
 * on click: update map data with the correct tile id on the corresponding coords
 */

fn get_event_pos(pos: (f64, f64)) -> (u8, u8) {
    let (x, y) = pos;
    ((x / BLOCK_SIZE as f64) as u8, (y / BLOCK_SIZE as f64) as u8)
}

fn format_event_pos(pos: (u8, u8)) -> String {
    let (x, y) = pos;
    format!("{:?}, {:?}", x, y)
}
