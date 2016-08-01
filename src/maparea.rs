extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate cairo;

use gtk::prelude::*;
use gdk::prelude::*;
use gtk::DrawingArea;
use gdk::EventMotion;
use gdk_pixbuf::{Pixbuf, InterpType};
use cairo::Context;

use std::cmp::{min,max};
use std::cell::RefCell;
use std::rc::Rc;

use constants::*;
use tileset::*;
use position::*;

#[derive(Clone, Debug)]
pub struct Maparea {
    mapset: Vec<u8>,
    tileset: Rc<RefCell<Tileset>>,
    width: u8,
    height: u8,
    hovered: Option<usize>,
    pix_cache: Pixbuf,
    palette: RgbPalette,
}

impl Maparea {
    pub fn new(width: u8, height: u8, mapset: Vec<u8>, tileset: Rc<RefCell<Tileset>>) -> Self {
        let pix_cache = Self::build_pix(width as i32 * BLOCK_SIZE as i32, height as i32 * BLOCK_SIZE as i32, &mapset, &*tileset.borrow());

        Maparea {
            mapset: mapset,
            tileset: tileset,
            width: width,
            height: height,
            hovered: None,
            pix_cache: pix_cache,
            palette: BASE_PALETTE,
        }
    }

    fn static_coords(index: usize, width: i32, height: i32) -> (i32, i32) {
        ((index as i32 * BLOCK_SIZE as i32) % width, (index as i32 * BLOCK_SIZE as i32 / width) * BLOCK_SIZE as i32)
    }

    pub fn coords(&self, index: usize) -> (i32, i32) {
        let width = Self::block_to_pixel_size(self.width);
        let height = Self::block_to_pixel_size(self.height);
        Self::static_coords(index, width, height)
    }

    fn block_to_pixel_size(size: u8) -> i32 {
        size as i32 * BLOCK_SIZE as i32
    }

    fn build_pix(width: i32, height: i32, mapset: &[u8], tileset: &Tileset) -> Pixbuf {
        Self::new_pixbuf_static(width, height, |context| {
            for (i, b_) in mapset.iter().enumerate() {
                let b = *b_;
                let (x, y) = Self::static_coords(i, width, height);
                let tile = tileset.get_tile_pix(b); //.new_subpixbuf(TILE_SIZE as i32 * (b % tileset_width), TILE_SIZE as i32 * ((b / tileset_width) as i32), TILE_SIZE as i32, TILE_SIZE as i32);

                context.set_source_pixbuf(&tile, x as f64, y as f64);
                context.paint();
            }
        })
    }

    fn new_pixbuf_static<F: FnOnce(&cairo::Context)>(width: i32, height: i32, call_on_context: F) -> Pixbuf {
        let mut surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height);
        {
            let context = cairo::Context::new(&surface);
            call_on_context(&context);
        }

        let mut data = Vec::with_capacity((width * height * 3) as usize);
        for b in surface.get_data().unwrap().iter().as_slice().chunks(4) {
            data.push(*b.get(2).unwrap());
            data.push(*b.get(1).unwrap());
            data.push(*b.get(0).unwrap());
        }

        Pixbuf::new_from_vec(data, 0, false, 8, width, height, width * 3)
    }

    fn paint(&self, context: &cairo::Context) {
        let (x0, y0, x1, y1) = context.clip_extents();
        let width = max(0, min(self.pix_cache.get_width(), x1 as i32) - x0 as i32);
        let height = max(0, min(self.pix_cache.get_height(), y1 as i32) - y0 as i32);

        let subpix = &self.pix_cache.new_subpixbuf(x0 as i32, y0 as i32, width, height);
        context.set_source_pixbuf(subpix, x0 as f64, y0 as f64);
        context.paint();

        if let Some(index) = self.hovered {
            self.paint_tile_with_palette(context, index, HOVER_PALETTE);
        }
    }

    fn paint_tile_with_palette(&self, context: &cairo::Context, index: usize, palette: RgbPalette) {
        let (x0, y0, x1, y1) = context.clip_extents();
        let width = max(0, min(self.pix_cache.get_width(), x1 as i32) - x0 as i32);
        let height = max(0, min(self.pix_cache.get_height(), y1 as i32) - y0 as i32);

        let (sel_x, sel_y) = self.coords(index);
        let sel_x1 = sel_x + BLOCK_SIZE as i32;
        let sel_y1 = sel_y + BLOCK_SIZE as i32;

        let within_left   = sel_x  >= x0 as i32 && sel_x  <= x1 as i32;
        let within_right  = sel_x1 >= x0 as i32 && sel_x1 <= x1 as i32;
        let within_top    = sel_y  >= y0 as i32 && sel_y  <= y1 as i32;
        let within_bottom = sel_y  >= y0 as i32 && sel_y  <= y1 as i32;

        if (within_left || within_right) && (within_top || within_bottom) {
            let x = max(sel_x, x0 as i32);
            let y = max(sel_y, y0 as i32);

            let width = min(sel_x1, x1 as i32) - x;
            let height = min(sel_y1, y1 as i32) - y;

            if width > 0 && height > 0 {
                let subpix = self.pix_cache.new_subpixbuf(x as i32, y as i32, width as i32, height as i32);
                let selected_subpix = change_palette(&subpix, self.palette, palette);
                context.set_source_pixbuf(&selected_subpix, x as f64, y as f64);
                context.paint();
            }
        }
    }

    pub fn hover_tile_at(&mut self, index: usize) {
        self.hovered = Some(index);
    }

    pub fn motion_notify(&mut self, el: &DrawingArea, ev: &gdk::EventMotion) {
        let (pos_x, pos_y) = ev.get_position();
        let block_x = pos_x as usize / BLOCK_SIZE;
        let block_y = pos_y as usize / BLOCK_SIZE;

        if block_x >= self.width as usize || block_y >= self.height as usize {
            return;
        }

        let new_hovered = block_x + block_y * self.width as usize;

        if let Some(old_hovered) = self.hovered {
            if new_hovered != old_hovered {
                println!("{:?}", (old_hovered, new_hovered));
                let (x, y) = self.coords(old_hovered);
                el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);

                self.hover_tile_at(new_hovered);
                let (x, y) = self.coords(new_hovered);
                el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
            }
        } else {
            self.hover_tile_at(new_hovered);
            let (x, y) = self.coords(new_hovered);
            el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
        }
    }

}

pub fn connect_events(maparea: Maparea, widget: &DrawingArea) {
    widget.add_events(drawing_area_mask_bits!());
    widget.set_size_request(maparea.width as i32 * BLOCK_SIZE as i32, maparea.height as i32 * BLOCK_SIZE as i32);

    let cell = Rc::new(RefCell::new(maparea));

    widget.connect_motion_notify_event(clone!(cell => move |el, ev| {
        cell.borrow_mut().motion_notify(el, ev);
        Inhibit::default()
    }));

    widget.connect_draw(clone!(cell => move |el, context| {
        cell.borrow_mut().paint(&context);
        Inhibit::default()
    }));
}