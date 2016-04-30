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
    let map_path = argv.nth(1).unwrap_or("../../../maps/pallettown.blk".to_string());
    let tileset_path = argv.nth(1).unwrap_or("../../../gfx/tilesets/overworld.t2.png".to_string());
    let blockset_path = argv.nth(2).unwrap_or("../../../gfx/blocksets/overworld.bst".to_string());

    let mut blockset: Vec<u8> = Vec::new();
    let mut blockset_file = File::open(&blockset_path).unwrap();
    blockset_file.read_to_end(&mut blockset);

    let mut mapset: Vec<u8> = Vec::new();
    let mut mapset_file = File::open(&map_path).unwrap();
    mapset_file.read_to_end(&mut mapset);

    let builder = Builder::new_from_file("builder.ui");
    let window: Window = builder.get_object("window").unwrap();
    let maparea: DrawingArea = builder.get_object("image").unwrap();
    let lbl_coords: Label = builder.get_object("lblCoords").unwrap();

    let pix = Pixbuf::new_from_file(&tileset_path).unwrap();
    let tileset: DrawingArea = builder.get_object("tileset").unwrap();
    maparea.add_events(drawing_area_mask_bits());
    tileset.add_events(drawing_area_mask_bits());

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

        lbl_coords.set_label(&format_event_pos(pos));
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

    let (maparea_w, maparea_h) = (10, 9);
    maparea.set_size_request(maparea_w * BLOCK_SIZE as i32, maparea_h * BLOCK_SIZE as i32);
    let mapcell = Rc::new(RefCell::new((mapset, blockset.clone())));
    let pix2 = pix.clone();

    maparea.connect_button_press_event(move |el, ev| {
        //el.queue_draw();
        Inhibit::default()
    });

    maparea.connect_draw(clone!(mapcell => move |el, context| {
        let (ref mut mapset, ref mut blockset) = *mapcell.borrow_mut();
        for (i, b_) in mapset.iter().enumerate() {
            let b = *b_ as usize;
            let coords: (u8, u8) = ((i % maparea_w as usize) as u8, (i / maparea_w as usize) as u8);
            draw_tile_block(context, &pix2, &blockset, b, coords);
        }

        context.set_source_pixbuf(&pix2, (maparea_w as usize * (BLOCK_SIZE + 2)) as f64, 0.);
        context.paint();
        Inhibit::default()
    }));


    tileset.set_size_request(blockset.len() as i32 * (TILE_SIZE / 4) as i32, BLOCK_SIZE as i32);
    tileset.connect_draw(clone!(cell => move |el, context| {
        let (ref mut x, ref mut y, ref mut selected, ref mut hovered) = *cell.borrow_mut();
        let tileset_width = pix.get_width() / 8 as i32;

        el.override_background_color(gtk::StateFlags::all(), &gdk_sys::GdkRGBA { red: 255., green: 1., blue: 0., alpha: 1. });
        context.paint();

        for (i, b_) in blockset.iter().enumerate() {
            let b = *b_ as i32;
            let mut tile = pix.new_subpixbuf(8 * (b % tileset_width), 8 * ((b / tileset_width) as i32), 8, 8);

            // tile = process_hover_and_select(&tile, *hovered, *selected); // XXX move this block in a function
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

pub fn draw_tile_block(context: &Context, pix: &Pixbuf, blockset: &Vec<u8>, index: usize, coords: (u8, u8)) {
    let (mut tile_bytes, _) = blockset.split_at(index * TILES_IN_BLOCK).1.split_at(16);

    let tileset_width = pix.get_width() / 8;
    let x0: f64 = (coords.0 as usize * BLOCK_SIZE) as f64;
    let y0: f64 = (coords.1 as usize * BLOCK_SIZE) as f64;

    for (i, b_) in tile_bytes.iter().enumerate() {
        let b = *b_;
        let tile = pix.new_subpixbuf(8 * (b as i32 % tileset_width), 8 * ((b as i32 / tileset_width) as i32), 8, 8)
            .scale_simple(TILE_SIZE as i32, TILE_SIZE as i32, InterpType::Nearest).unwrap();

        context.set_source_pixbuf(&tile, x0 + ((i % 4) * TILE_SIZE) as f64, y0 + (((i / 4) as i32) * TILE_SIZE as i32) as f64);
        context.paint();
    }
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
