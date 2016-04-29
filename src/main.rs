mod rustmap;
mod constants;

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
use gtk::{Builder, DrawingArea, Label, Window};
use gdk::prelude::*;
use gdk::{Gravity, EventTouch, EventType, EventMask};
use gdk_pixbuf::{Pixbuf, InterpType};
use cairo::Context;

use constants::*;

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

fn main () {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let mut argv = std::env::args();
    let tileset_path = argv.nth(1).unwrap_or("../../../gfx/tilesets/overworld.t2.png".to_string());
    let blockset_path = argv.nth(2).unwrap_or("../../../gfx/blocksets/overworld.bst".to_string());

    let mut blockset: Vec<u8> = Vec::new();
    let mut blockset_file = File::open(&blockset_path).unwrap();
    blockset_file.read_to_end(&mut blockset);

    let builder = Builder::new_from_file("builder.ui");
    let window: Window = builder.get_object("window").unwrap();
    let image: DrawingArea = builder.get_object("image").unwrap();
    let lblCoords: Label = builder.get_object("lblCoords").unwrap();

    let pix = Pixbuf::new_from_file(&tileset_path).unwrap();
    let tileset: DrawingArea = builder.get_object("tileset").unwrap();
    let mask = gdk_sys::GDK_POINTER_MOTION_MASK
        | gdk_sys::GDK_BUTTON_PRESS_MASK
        | gdk_sys::GDK_BUTTON1_MOTION_MASK
        // | gdk_sys::GDK_ENTER_NOTIFY_MASK // don't need this for the moment
        | gdk_sys::GDK_LEAVE_NOTIFY_MASK;
    tileset.add_events(mask.bits() as i32);

    let surface = cairo::ImageSurface::create(cairo::Format::Rgb24, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
    let context = cairo::Context::new(&surface);

    let x: u8 = 0;
    let selected: u8 = 0;
    let hovered: Option<u8> = None;
    let y: u8 = 0;
    let cell = Rc::new(RefCell::new((x, y, selected, hovered)));

    // UI initialization.
    window.set_gravity(Gravity::Center);
    window.show_all();

    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    tileset.connect_leave_notify_event(clone!(cell => move |el, ev| {
        let (_, _, _, ref mut hovered) = *cell.borrow_mut();
        if let Some(hovered_inner) = *hovered {
            redraw_tile(el, hovered_inner as usize);
        }
        *hovered = None;
        Inhibit::default()
    }));

    tileset.connect_motion_notify_event(clone!(cell => move |el, ev| {
        let (ref mut x, _, _, ref mut hovered) = *cell.borrow_mut();
        let pos = get_event_pos(ev.get_position());
        let (lx, ly) = pos;
        *x = lx;
        if let Some(hovered_inner) = *hovered {
            if lx != hovered_inner {
                redraw_tile(el, hovered_inner as usize);
            }
        }
        *hovered = Some(*x);
        redraw_tile(el, *x as usize);

        lblCoords.set_label(&format_event_pos(pos));
        Inhibit::default()
    }));

    image.connect_button_press_event(clone!(cell => move |el, ev| {
        Inhibit::default()
    }));

    tileset.connect_button_press_event(clone!(cell => move|el, ev| {
        let (ref mut x, _, ref mut selected, _) = *cell.borrow_mut();
        let pos = get_event_pos(ev.get_position());
        let (lx, _) = pos;
        *x = lx;
        if lx != *selected {
            redraw_tile(el, *selected as usize);
            *selected = *x;
            redraw_tile(el, *selected as usize);
        }

        Inhibit::default()
    }));

    tileset.set_size_request(blockset.len() as i32 * (TILE_SIZE / 4) as i32, BLOCK_SIZE as i32);
    tileset.connect_draw(clone!(cell => move |el, context| {
        let tileset_width = pix.get_width() / 8 as i32;

        let (ref mut x, ref mut y, ref mut selected, ref mut hovered) = *cell.borrow_mut();

        el.override_background_color(gtk::StateFlags::all(), &gdk_sys::GdkRGBA { red: 255., green: 1., blue: 0., alpha: 1. });
        context.paint();

        for (i, b_) in blockset.iter().enumerate() {
            let b = *b_ as i32;
            let mut tile = pix.new_subpixbuf(8 * (b % tileset_width), 8 * ((b / tileset_width) as i32), 8, 8);

            if hovered.is_some() && hovered.map_or(false, |hovered_inner| { (i / TILES_IN_BLOCK) as usize == hovered_inner as usize }) {
                let mut pxs: Vec<u8> = Vec::new();
                unsafe {
                    for (i, b) in tile.get_pixels().iter().enumerate() {
                        let b1: u8 = match *b {
                            170 => match i % 3 {
                                0 => 60,
                                1 => 120,
                                _ => 140,
                            },
                            85 => match i % 3 {
                                0 => 100,
                                1 => 20,
                                _ => 50,
                            },
                            255 => match i % 3 {
                                0 => 255,
                                1 => 255,
                                _ => 100,
                            },
                            x => x
                        };
                        pxs.push(b1);
                    }
                };
                let ctile = Pixbuf::new_from_vec(pxs, tile.get_colorspace(), false, tile.get_bits_per_sample(), tile.get_width(), tile.get_height(), tile.get_rowstride());
                tile = ctile;
            } else if (i / TILES_IN_BLOCK) as usize == *selected as usize {
                let mut pxs: Vec<u8> = Vec::new();
                unsafe {
                    for (i, b) in tile.get_pixels().iter().enumerate() {
                        let b1: u8 = match *b {
                            170 => match i % 3 {
                                0 => 60,
                                1 => 180,
                                _ => 60,
                            },
                            85 => match i % 3 {
                                0 => 20,
                                1 => 50,
                                _ => 80,
                            },
                            255 => match i % 3 {
                                0 => 220,
                                1 => 250,
                                _ => 140,
                            },
                            x => x
                        };
                        pxs.push(b1);
                    }
                };
                let ctile = Pixbuf::new_from_vec(pxs, tile.get_colorspace(), false, tile.get_bits_per_sample(), tile.get_width(), tile.get_height(), tile.get_rowstride());
                tile = ctile;
            }
            tile = tile.scale_simple(TILE_SIZE as i32, TILE_SIZE as i32, InterpType::Nearest).unwrap();

            context.set_source_pixbuf(&tile, (((i % 4) * TILE_SIZE) + (i / 16) * BLOCK_SIZE) as f64, ((((i / 4) % 4) as i32) * TILE_SIZE as i32) as f64);
            context.paint();
        }
        context.save();
        Inhibit::default()
    }));

    // Run the main loop.
    gtk::main();
}

fn redraw_tile<W: gtk::WidgetExt>(el: &W, index: usize) {
    el.queue_draw_area((index * BLOCK_SIZE) as i32, 0, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
}

fn get_event_pos(pos: (f64, f64)) -> (u8, u8) {
    let (x, y) = pos;
    ((x / BLOCK_SIZE as f64) as u8, (y / BLOCK_SIZE as f64) as u8)
}

fn format_event_pos(pos: (u8, u8)) -> String {
    let (x, y) = pos;
    format!("{:?}, {:?}", x, y)
}
