use gtk::gio::MenuModel;
use gtk::prelude::*;
use gtk::{Application, Builder, PopoverMenuBar, Window};

pub fn build_ui(application: &Application) {
    let ui_src = include_str!("ui/narrative_director.cmb.ui");
    let builder = Builder::new();
    builder
        .add_from_string(ui_src)
        .expect("Couldn't add from string");

    let window: Window = builder.object("window").expect("Couldn't get window");
    window.set_application(Some(application));

    let menubar: PopoverMenuBar = builder.object("popup_menu").expect("Couldn't get menubar.");
    let menumodel: MenuModel = builder
        .object("menuModel")
        .expect("Couldn't get menu model.");

    menubar.set_menu_model(Some(&menumodel));

    window.show();
}

fn main() {
    let application = Application::new(
        Some("org.divarktech.narrative_director"),
        Default::default(),
    );
    application.connect_activate(build_ui);
    application.run();
}
