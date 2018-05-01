use std::io::prelude::*;
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::PathBuf;

use gtk;
use gdk;
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
    File::open(&path)
        .map(|mut f| {
            let _ = f.read_to_end(&mut bytes);
            bytes
        })
        .ok()
}


pub struct Gui {
    config: Rc<RefCell<Config>>,
    builder: Builder,
    window: Rc<RefCell<Window>>,
    maparea: Rc<RefCell<Option<Maparea>>>,
}

impl Gui {
    pub fn new(config: Option<Config>, builder: Builder) -> Self {
        let window: Window = builder.get_object("window").expect(
            "No window found in builder",
        );

        Gui {
            config: Rc::new(RefCell::new(config.unwrap_or_default())),
            builder: builder,
            window: Rc::new(RefCell::new(window)),
            maparea: Rc::new(RefCell::new(None)),
        }
    }

    fn init_window(&self) {
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

    fn init_menu(&self) {
        let save_as: MenuItem = self.builder.get_object("menu_save_as").unwrap();
        save_as.add_events(drawing_area_mask_bits!());

        let ref window_cell = self.window;
        let ref maparea_cell = self.maparea;
        let config_cell = self.config.clone();

        //
        // SAVE
        //
        save_as.connect_activate(clone!(window_cell, maparea_cell => move |_| {
            Gui::save_map_as(&maparea_cell.borrow(), &mut config_cell.borrow_mut(), &window_cell.borrow());
        }));

        let save: MenuItem = self.builder.get_object("menu_save").unwrap();
        save.add_events(drawing_area_mask_bits!());

        let config_cell = self.config.clone();

        save.connect_activate(clone!(window_cell, maparea_cell, config_cell => move |_| {
            /* keep filename for future use */
            let mut config = config_cell.borrow_mut();
            let _ = config.recent.as_ref()
                .and_then(|recent| recent.map_path.clone())
                .map(|map_path| {
                    Gui::save_map(&maparea_cell.borrow(), map_path)
                })
                .ok_or_else(|| {
                    Gui::save_map_as(&maparea_cell.borrow(), &mut config, &window_cell.borrow());
                    //config_cell.borrow_mut().recent.as_mut().map(|mut recent| recent.map_path = Some(filename));
                });
        }));

        //
        // UNDO & REDO
        //
        let undo: MenuItem = self.builder.get_object("menu_undo").unwrap();
        undo.add_events(drawing_area_mask_bits!());

        undo.connect_activate(clone!(maparea_cell => move |_| {
            /* TODO pop last action from history */
            maparea_cell.borrow_mut().as_mut().map(|maparea| maparea.undo());
        }));

        let redo: MenuItem = self.builder.get_object("menu_redo").unwrap();
        redo.add_events(drawing_area_mask_bits!());

        redo.connect_activate(clone!(maparea_cell => move |_| {
            /* TODO push next action from history */
            maparea_cell.borrow_mut().as_mut().map(|maparea| maparea.redo());
        }));

    }

    fn save_map_as(maparea: &Option<Maparea>, config: &mut Config, window: &Window) {
        let file_dialog = gtk::FileChooserDialog::new(
            Some("Save As"),
            Some(window),
            gtk::FileChooserAction::Save,
        );
        file_dialog.add_button("OK", gtk::ResponseType::Ok.into());
        file_dialog.add_button("Cancel", gtk::ResponseType::Cancel.into());
        let response = file_dialog.run();
        let filename = file_dialog.get_filename();
        file_dialog.destroy();

        if response == gtk::ResponseType::Ok.into() {
            let filename = filename.expect("filename is missing");

            /* keep filename for future use */
            Gui::save_map(maparea, filename.clone());

            config.recent.as_mut().map(|recent| {
                recent.map_path = Some(filename)
            });
        }
    }

    fn save_map(maparea: &Option<Maparea>, filename: PathBuf) {
        maparea.as_ref().map(|maparea| {
            maparea.on_bytes(|bytes| {
                let written_bytes = File::create(&filename).and_then(|mut f| f.write(bytes));
                println!("{:?}", written_bytes);
            });
        });
    }

    pub fn load_map(&self, filename: &PathBuf) -> Vec<u8> {
        let mut mapset: Vec<u8> = Vec::new();
        let mut mapset_file = File::open(filename).expect(&format!("Invalid path: {:?}", filename));
        let result = mapset_file.read_to_end(&mut mapset);
        if let Err(err) = result {
            println!("{}", err);
        }

        mapset
    }

    pub fn run(&mut self) {
        {
            let config = self.config.borrow();
            let map_height = config.recent.as_ref().unwrap().map_height.unwrap();
            let map_width = config.recent.as_ref().unwrap().map_width.unwrap();
            let mapset = config
                .recent
                .as_ref()
                .and_then({
                    |recent| recent.map_path.as_ref()
                })
                .map(|map_path| self.load_map(map_path))
                .expect("Error on loading mapset");

            let tileset_path = config
                .recent
                .as_ref()
                .unwrap()
                .tileset_path
                .as_ref()
                .expect("No tileset_path provided");
            let blockset_path = config
                .recent
                .as_ref()
                .unwrap()
                .blockset_path
                .as_ref()
                .expect("No blockset_path provided");

            let lbl_coords: Label = self.builder.get_object("lblCoords").expect(
                "No lblCoords found in builder",
            );
            let tileset_widget: DrawingArea = self.builder.get_object("tileset").expect(
                "No tileset found in builder",
            );
            let maparea_widget: DrawingArea = self.builder.get_object("maparea").expect(
                "No maparea found in builder",
            );

            // TODO need better error handling
            let blockset: Vec<u8> = get_bytes_from_filepath(blockset_path).unwrap();
            // TODO need better error handling
            let tileset_pix = Pixbuf::new_from_file(tileset_path).unwrap();
            let tileset = Tileset::from_data(tileset_widget, &blockset, &tileset_pix);
            tileset.borrow_mut().select_tile_at(0);

            self.maparea =
                Maparea::from_data(maparea_widget, map_width, map_height, mapset, tileset);
            self.maparea.borrow().as_ref().map(|maparea| {
                let lbl_coords = lbl_coords.clone();
                maparea.widget.connect_motion_notify_event(move |_, ev| {
                    let pos = get_event_pos(ev.get_position());
                    lbl_coords.set_label(&format!("{:?}", pos));
                    Inhibit::default()
                });
            });
            Maparea::connect_events(&self.maparea);
        }

        // Menu
        self.init_menu();

        // UI initialization.
        self.init_window();

        // Run the main loop.
        gtk::main();
    }
}
