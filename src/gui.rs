use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

use gtk;
use gtk::prelude::*;
use gtk::{Builder, DrawingArea, Label, MenuItem, Window};
use gdk::Gravity;
use gdk_pixbuf::Pixbuf;
use gdk_sys;

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
    maparea: Rc<RefCell<Option<Maparea>>>,
}

impl Gui {
    pub fn new(config: Config, builder: Builder) -> Self {
        let window: Window = builder.get_object("window").expect("No window found in builder");

        Gui {
            config: config,
            builder: builder,
            window: Rc::new(RefCell::new(window)),
            maparea: Rc::new(RefCell::new(None)),
        }
    }

    fn init_ui(&self) {
        let window = self.window.borrow();
        window.set_gravity(Gravity::Center);
        window.set_title("Rustmap v0.1.0");
        window.show_all();

        // Handle closing of the window.
        window.connect_delete_event(|_, _| {
            gtk::main_quit();
            Inhibit(false)
        });
    }

    pub fn load_map(&self, filename: &str) -> Vec<u8> {
        let mut mapset: Vec<u8> = Vec::new();
        let mut mapset_file = File::open(filename).expect(&format!("Invalid map_path {}", filename));
        let result = mapset_file.read_to_end(&mut mapset);
        if let Err(err) = result {
            println!("{}", err);
        }

        mapset
    }

    pub fn run(&mut self) {
        let mapset: Vec<u8> = self.config.recent.as_ref().and_then({
            |recent| recent.map_path.as_ref()
        }).map(|map_path| {
            self.load_map(&map_path)
        }).unwrap();

        let tileset_path = self.config.recent.as_ref().unwrap().tileset_path.as_ref().expect("No tileset_path provided");
        let blockset_path = self.config.recent.as_ref().unwrap().blockset_path.as_ref().expect("No blockset_path provided");

        let lbl_coords: Label = self.builder.get_object("lblCoords").expect("No lblCoords found in builder");
        let tileset_widget: DrawingArea = self.builder.get_object("tileset").expect("No tileset found in builder");
        let maparea_widget: DrawingArea = self.builder.get_object("maparea").expect("No maparea found in builder");

        let blockset: Vec<u8> = get_bytes_from_filepath(blockset_path).unwrap();
        let tileset_pix = Pixbuf::new_from_file(tileset_path).unwrap();
        let tileset = Tileset::from_data(tileset_widget, &blockset, &tileset_pix);
        tileset.borrow_mut().select_tile_at(0);

        self.maparea = Maparea::from_data(maparea_widget, 20, 18, mapset, tileset);

        self.maparea.borrow().as_ref().map(|maparea| {
            let lbl_coords = lbl_coords.clone();
            maparea.widget.connect_motion_notify_event(move |_, ev| {
                let pos = get_event_pos(ev.get_position());
                lbl_coords.set_label(&format!("{:?}", pos));
                Inhibit::default()
            });
        });

        // Menu
        //self.init_menu();
        let save_as: MenuItem = self.builder.get_object("menu_save_as").unwrap();
        save_as.add_events(drawing_area_mask_bits!());

        let ref window_cell = self.window;
        let ref maparea_cell = self.maparea;
        save_as.connect_activate(clone!(window_cell, maparea_cell => move |_| {
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
                maparea_cell.borrow().as_ref().map(|maparea| {
                    maparea.on_bytes(move |bytes| {
                        let written_bytes = File::create(&filename).and_then(|mut f| f.write(bytes));
                        println!("{:?}", written_bytes);
                    });
                });
            }
        }));

        // UI initialization.
        self.init_ui();

        // Run the main loop.
        gtk::main();
    }
}
