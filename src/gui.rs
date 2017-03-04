extern crate gtk;
extern crate gdk;
extern crate gdk_sys;
extern crate gdk_pixbuf;
extern crate cairo;

use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{Builder, DrawingArea, Label, MenuItem, Window};
use gdk::Gravity;
use gdk_pixbuf::Pixbuf;

use config::Config;
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


pub struct Gui {
    config: Config,
    builder: Builder,
    window: Rc<RefCell<Window>>,
}

impl Gui {
    pub fn new(config: Config, builder: Builder) -> Self {
        let window: Window = builder.get_object("window").expect("No window found in builder");

        Gui {
            config: config,
            builder: builder,
            window: Rc::new(RefCell::new(window)),
        }
    }

    fn init_ui(&self) {
        let window = self.window.borrow();
        window.set_gravity(Gravity::Center);
        window.show_all();

        // Handle closing of the window.
        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
    }

    pub fn run(&self) {
        let map_path = self.config.recent.map_path.as_ref().expect("No map provided");
        let tileset_path = self.config.recent.tileset_path.as_ref().expect("No tileset provided");
        let blockset_path = self.config.recent.blockset_path.as_ref().expect("no blockset provided");

        let mut mapset: Vec<u8> = Vec::new();
        let mut mapset_file = File::open(map_path).unwrap();
        if let Err(err) = mapset_file.read_to_end(&mut mapset) {
            println!("{}", err);
            return;
        }

        let lbl_coords: Label = self.builder.get_object("lblCoords").unwrap();
        let tileset_widget: DrawingArea = self.builder.get_object("tileset").unwrap();
        let maparea_widget: DrawingArea = self.builder.get_object("maparea").unwrap();

        let blockset: Vec<u8> = get_bytes_from_filepath(blockset_path).unwrap();
        let tileset_pix = Pixbuf::new_from_file(tileset_path).unwrap();
        let tileset = Tileset::from_data(tileset_widget, &blockset, &tileset_pix);
        tileset.borrow_mut().select_tile_at(0);

        let maparea = Maparea::from_data(maparea_widget, 20, 18, mapset, tileset);

        maparea.borrow().widget.connect_button_press_event(move |_, ev| {
            let pos = get_event_pos(ev.get_position());
            lbl_coords.set_label(&format!("{:?}", pos));

            Inhibit::default()
        });

        // Menu
        //self.init_menu();
        let save_as: MenuItem = self.builder.get_object("menu_save_as").unwrap();
        save_as.add_events(drawing_area_mask_bits!());

        let ref window_cell = self.window;
        save_as.connect_activate(clone!(window_cell => move |_| {
            let file_dialog = gtk::FileChooserDialog::new(
                Some("Save As"), Some(&*window_cell.borrow()), gtk::FileChooserAction::Save);
            file_dialog.add_button("OK", gtk::ResponseType::Ok.into());
            file_dialog.add_button("Cancel", gtk::ResponseType::Cancel.into());
            let response = file_dialog.run();
            let filename = file_dialog.get_filename();
            file_dialog.destroy();

            if response == gtk::ResponseType::Ok.into() {
                let filename = filename.expect("filename is missing");
                /* save file */
                maparea.borrow().on_bytes(move |bytes| {
                    let written_bytes = File::create(&filename).and_then(|mut f| f.write(bytes));
                    println!("{:?}", written_bytes);
                });
            }
        }));

        // UI initialization.
        self.init_ui();

        // Run the main loop.
        gtk::main();
    }
}