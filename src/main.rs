mod rustmap;

extern crate gtk;
extern crate gdk;

use gtk::prelude::*;
use gtk::{Builder, Button, ButtonsType, DialogFlags, DrawingArea, Label, MessageType, MessageDialog, Window, WindowType};
use gdk::Gravity;

fn main () {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let builder = Builder::new_from_file("builder.ui");
    let window: Window = builder.get_object("window").unwrap();
    let area: DrawingArea = builder.get_object("mapArea").unwrap();
    let lblCoords: Label = builder.get_object("lblCoords").unwrap();

    // UI initialization.
    window.set_title("First GTK+ Program");
    window.set_gravity(Gravity::Center);
    window.set_default_size(350, 70);
    // Don't forget to make all widgets visible.
    window.show_all();

    // Handle closing of the window.
    window.connect_delete_event(|_, _| {
        // Stop the main loop.
        gtk::main_quit();
        // Let the default handler destroy the window.
        Inhibit(false)
    });

    area.connect_motion_notify_event(move |_, ev| {
        let (x, y) = ev.get_position();
        let pos = format!("{:?}, {:?}", (x / 64.) as u8, (y / 64.) as u8);
        lblCoords.set_label(&pos);
        Inhibit::default()
    });
    // button.connect_clicked(|_| {
    //     println!("Clicked!");
    // });

    // Run the main loop.
    gtk::main();
}
