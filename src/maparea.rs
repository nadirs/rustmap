use gtk::prelude::*;
use gdk::prelude::*;
use gtk::DrawingArea;
use gdk;
use gdk_pixbuf::Pixbuf;
use gdk_sys;
use cairo;
use cairo::Context;

use std::cmp::{min,max};
use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;
use std::mem;

use constants::*;
use tileset::Tileset;
use palette::change_palette;
use history::History;

#[derive(Clone, Debug)]
pub struct Maparea {
    mapset: Vec<u8>,
    tileset: Rc<RefCell<Tileset>>,
    width: u8,
    height: u8,
    hovered: Option<usize>,
    pix_cache: Pixbuf,
    palette: RgbPalette,
    pub widget: DrawingArea,
    history: History,
}

impl Maparea {
    pub fn new(widget: DrawingArea, width: u8, height: u8, mapset: Vec<u8>, tileset: Rc<RefCell<Tileset>>) -> Self {
        let pix_cache = Self::static_build_pix(width as i32 * BLOCK_SIZE as i32, height as i32 * BLOCK_SIZE as i32, &mapset, &*tileset.borrow());

        let history = History::new(mapset.clone());

        Maparea {
            mapset: mapset,
            tileset: tileset,
            width: width,
            height: height,
            hovered: None,
            pix_cache: pix_cache,
            palette: BASE_PALETTE,
            widget: widget,
            history: history,
        }
    }

    pub fn on_bytes<T, F: Fn(&[u8]) -> T>(&self, call_on_bytes: F) -> T {
        call_on_bytes(&self.mapset)
    }

    pub fn from_data(widget: DrawingArea, width: u8, height: u8, mapset: Vec<u8>, tileset: Rc<RefCell<Tileset>>) -> Rc<RefCell<Option<Self>>> {
        widget.add_events(drawing_area_mask_bits!());
        widget.set_size_request(width as i32 * BLOCK_SIZE as i32, height as i32 * BLOCK_SIZE as i32);

        let maparea = Maparea::new(widget, 20, 18, mapset, tileset);
        let cell = Rc::new(RefCell::new(Some(maparea)));

        cell
    }

    pub fn connect_events(cell: &Rc<RefCell<Option<Self>>>) {
        let cell_maparea: Ref<Option<Self>> = cell.borrow();
        cell_maparea.as_ref().map(|maparea|{
            let ref widget = maparea.widget;

            widget.connect_motion_notify_event(clone!(cell => move |el, ev| {
                cell.borrow_mut().as_mut().unwrap().motion_notify(el, ev);
                Inhibit::default()
            }));

            widget.connect_button_press_event(clone!(cell => move|el, ev| {
                cell.borrow_mut().as_mut().unwrap().button_press(el, ev);
                Inhibit::default()
            }));

            widget.connect_draw(clone!(cell => move |_, context| {
                cell.borrow_mut().as_mut().unwrap().paint(&context);
                Inhibit::default()
            }));
        });
    }

    fn set_mapset(&mut self, mapset: Vec<u8>) -> Vec<u8> {
        let old_mapset = mem::replace(&mut self.mapset, mapset);
        self.widget.queue_draw_area(0, 0, self.width as i32 * BLOCK_SIZE as i32, self.height as i32 * BLOCK_SIZE as i32);
        old_mapset
    }

    fn replace_mapset(&mut self, state: Vec<u8>) {
        for (index, block) in Maparea::diff(&self.mapset, &state) {
            self.update_map_block(index, block);
        }
        self.set_mapset(state);
    }

    pub fn redo(&mut self) {
        self.history.redo().map(|state| self.replace_mapset(state));
    }

    pub fn undo(&mut self) {
        self.history.undo().map(|state| self.replace_mapset(state));
    }

    /// Find which bytes changed
    /// # Example:
    ///
    /// ```
    /// let old = vec![0, 1, 2];
    /// let new = vec![0, 2, 2];
    /// diff(old, new);
    /// ```
    fn diff(old: &[u8], new: &[u8]) -> Vec<(usize, u8)> {
        let mut result = Vec::new();

        for (i, (o, n)) in old.iter().zip(new).enumerate() {
            if o != n {
                result.push((i, *n));
            }
        }

        result
    }

    fn static_coords(index: usize, width: i32, _: i32) -> (i32, i32) {
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

    fn static_build_pix(width: i32, height: i32, mapset: &[u8], tileset: &Tileset) -> Pixbuf {
        Self::new_pixbuf_static(width, height, |context| {
            for (i, b_) in mapset.iter().enumerate() {
                let b = *b_;
                let (x, y) = Self::static_coords(i, width, height);
                let tile = tileset.get_tile_pix(b);

                context.set_source_pixbuf(&tile, x as f64, y as f64);
                context.paint();
            }
        })
    }

    fn new_pixbuf_static<F: FnOnce(&cairo::Context)>(width: i32, height: i32, call_on_context: F) -> Pixbuf {
        let mut surface = cairo::ImageSurface::create(cairo::Format::Rgb24, width, height).expect("Error in new_pixbuf_static: cannot create ImageSurface");
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

    pub fn update_map_block(&mut self, map_index: usize, block_index: u8) {
        self.mapset[map_index] = block_index;
        let width = Self::block_to_pixel_size(self.width);
        let height = Self::block_to_pixel_size(self.height);

        let pix_cache = Self::new_pixbuf_static(width, height, |context: &Context| {
            context.set_source_pixbuf(&self.pix_cache, 0., 0.);
            context.paint();

            let (x, y) = self.coords(map_index);
            context.set_source_pixbuf(&self.tileset.borrow().get_tile_pix(block_index), x as f64, y as f64);
            context.paint();
        });

        self.pix_cache = pix_cache;
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

    pub fn button_press(&mut self, el: &DrawingArea, ev: &gdk::EventButton) {
        let (pos_x, pos_y) = ev.get_position();
        let block_x = pos_x as usize / BLOCK_SIZE;
        let block_y = pos_y as usize / BLOCK_SIZE;

        if block_x >= self.width as usize || block_y >= self.height as usize {
            return;
        }

        let block_index = block_x + block_y * self.width as usize;
        assert!(block_index < self.mapset.len());

        match ev.as_ref().button {
            1 => self.button_press_left(el, block_index),
            3 => self.button_press_right(el, block_index),
            _ => (),
        }
    }

    pub fn button_press_left(&mut self, el: &DrawingArea, block_index: usize) {
        let selected_block = self.tileset.borrow().selected;
        if let Some(selected_block_index) = selected_block {
            self.update_map_block(block_index, selected_block_index);
            self.history.update(self.mapset.clone());
            let (x, y) = self.coords(block_index);
            el.queue_draw_area(x as i32, y as i32, BLOCK_SIZE as i32, BLOCK_SIZE as i32);
        }
    }

    pub fn button_press_right(&mut self, _: &DrawingArea, block_index: usize) {
        let tile_index = self.mapset[block_index];
        self.tileset.borrow_mut().select_tile_at(tile_index);
    }
}
