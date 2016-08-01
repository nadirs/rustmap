extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate cairo;

use gtk::prelude::*;
use gdk::prelude::*;
use gtk::DrawingArea;
use gdk_pixbuf::Pixbuf;

use std::cmp::{min,max};
use std::cell::RefCell;
use std::rc::Rc;

use constants::*;
use position::*;


#[derive(Clone, Debug)]
pub struct Tileset {
    width: i32,
    height: i32,
    selected: Option<u8>,
    hovered: Option<u8>,
    pix_cache: Pixbuf,
    palette: RgbPalette,
}

impl Tileset {
    pub fn new(width: i32, height: i32, pix: &Pixbuf, blockset: &[u8]) -> Self {
        let tileset_pix_cache = Self::build_tileset_pix(width, height, pix, blockset);
        Tileset {
            width: width,
            height: height,
            selected: None,
            hovered: None,
            pix_cache: tileset_pix_cache,
            palette: BASE_PALETTE,
        }
    }

    pub fn from_data(blockset: &Vec<u8>, widget: &DrawingArea, pix: &Pixbuf) -> Rc<RefCell<Self>> {
        widget.add_events(drawing_area_mask_bits!());

        let width = blockset.len() as i32 * (TILE_SIZE as i32 / 4) as i32;
        let height = BLOCK_SIZE as i32;
        widget.set_size_request(width, height);

        let tileset = Tileset::new(width, height, pix, blockset);
        let cell = Rc::new(RefCell::new(tileset));

        widget.connect_leave_notify_event(clone!(cell => move |el, ev| {
            cell.borrow_mut().leave_notify(el, ev);
            Inhibit::default()
        }));

        widget.connect_motion_notify_event(clone!(cell => move |el, ev| {
            cell.borrow_mut().motion_notify(el, ev);
            Inhibit::default()
        }));

        widget.connect_button_press_event(clone!(cell => move|el, ev| {
            cell.borrow_mut().button_press(el, ev);
            Inhibit::default()
        }));

        widget.connect_draw(clone!(cell => move |el, context| {
            cell.borrow_mut().paint(&context);
            Inhibit::default()
        }));

        cell
    }

    fn build_tileset_pix(width: i32, height: i32, pix: &Pixbuf, blockset: &[u8]) -> Pixbuf {
        Self::new_pixbuf_static(width, height, |context| {
            let tileset_width = pix.get_width() / TILE_SIZE as i32;
            for (i, b_) in blockset.iter().enumerate() {
                let b = *b_ as i32;
                let tile = pix.new_subpixbuf(TILE_SIZE as i32 * (b % tileset_width), TILE_SIZE as i32 * ((b / tileset_width) as i32), TILE_SIZE as i32, TILE_SIZE as i32);

                context.set_source_pixbuf(&tile, (((i % 4) * TILE_SIZE) + (i / 16) * BLOCK_SIZE) as f64, ((((i / 4) % 4) as i32) * TILE_SIZE as i32) as f64);
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

    pub fn coords(&self, index: u8) -> (usize, usize) {
        ((index as usize * BLOCK_SIZE) % self.width as usize, (index as usize * BLOCK_SIZE) / self.width as usize)
    }

    pub fn select_tile_at(&mut self, index: u8) {
        self.selected = Some(index);
    }

    pub fn hover_tile_at(&mut self, index: u8) {
        self.hovered = Some(index);
    }

    pub fn get_tile_pix(&self, index: u8) -> Pixbuf {
        let (x, y) = self.coords(index);
        self.pix_cache.new_subpixbuf(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32)
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

        if let Some(index) = self.selected {
            self.paint_tile_with_palette(context, index, SELECT_PALETTE);
        }
    }

    fn paint_tile_with_palette(&self, context: &cairo::Context, index: u8, palette: RgbPalette) {
        let (x0, y0, x1, y1) = context.clip_extents();
        let width = max(0, min(self.pix_cache.get_width(), x1 as i32) - x0 as i32);
        let height = max(0, min(self.pix_cache.get_height(), y1 as i32) - y0 as i32);

        let (sel_x, sel_y) = self.coords(index);
        let sel_x1 = sel_x + BLOCK_SIZE;
        let sel_y1 = sel_y + BLOCK_SIZE;

        let within_left   = sel_x  >= x0 as usize && sel_x  <= x1 as usize;
        let within_right  = sel_x1 >= x0 as usize && sel_x1 <= x1 as usize;
        let within_top    = sel_y  >= y0 as usize && sel_y  <= y1 as usize;
        let within_bottom = sel_y  >= y0 as usize && sel_y  <= y1 as usize;

        if (within_left || within_right) && (within_top || within_bottom) {
            let x = max(sel_x, x0 as usize);
            let y = max(sel_y, y0 as usize);

            let width = min(sel_x1, x1 as usize) - x;
            let height = min(sel_y1, y1 as usize) - y;

            if width > 0 && height > 0 {
                let subpix = self.pix_cache.new_subpixbuf(x as i32, y as i32, width as i32, height as i32);
                let selected_subpix = change_palette(&subpix, self.palette, palette);
                context.set_source_pixbuf(&selected_subpix, x as f64, y as f64);
                context.paint();
            }
        }
    }

    pub fn leave_notify(&mut self, el: &DrawingArea, _ev: &gdk::EventCrossing) {
        if let Some(old_hovered) = self.hovered {
            let (x, y) = self.coords(old_hovered);
            el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
        }
        self.hovered = None;
    }

    pub fn motion_notify(&mut self, el: &DrawingArea, ev: &gdk::EventMotion) {
        let (lx, _) = get_event_pos(ev.get_position());

        if let Some(old_hovered) = self.hovered {
            if lx != old_hovered {
                println!("{:?}", (old_hovered, lx));
                let (x, y) = self.coords(old_hovered);
                el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);

                self.hover_tile_at(lx);
                let (x, y) = self.coords(lx);
                el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
            }
        } else {
            self.hover_tile_at(lx);
            let (x, y) = self.coords(lx);
            el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
        }
    }

    pub fn button_press(&mut self, el: &DrawingArea, ev: &gdk::EventButton) {
        let pos = get_event_pos(ev.get_position());
        let (lx, _) = pos;

        let should_select = self.selected.map_or(true, |old_selected| { lx != old_selected });
        if should_select {
            self.selected.map(|old_selected| {
                let (x, y) = self.coords(old_selected);
                el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
            });

            self.select_tile_at(lx);
            let (x, y) = self.coords(lx);
            el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
        }
    }
}

pub fn change_palette(tile: &Pixbuf, from_pal: RgbPalette, to_pal: RgbPalette) -> Pixbuf {
    let mut pxs: Vec<u8> = Vec::new();
    unsafe {
        for chunk in tile.get_pixels().iter().as_slice().chunks(3) {
            let triple = rgb_triple_from(chunk);
            let (red, green, blue) = {
                if triple == from_pal.0 {
                    to_pal.0
                } else if triple == from_pal.1 {
                    to_pal.1
                } else if triple == from_pal.2 {
                    to_pal.2
                } else if triple == from_pal.3 {
                    to_pal.3
                } else {
                    triple //panic!("{:?} match not found: {:?} {:?} {:?}", i, triple, from_pal, to_pal);
                }
            };

            pxs.push(red);
            pxs.push(green);
            pxs.push(blue);
        }
    };
    Pixbuf::new_from_vec(pxs, tile.get_colorspace(), false, tile.get_bits_per_sample(), tile.get_width(), tile.get_height(), tile.get_rowstride())
}
