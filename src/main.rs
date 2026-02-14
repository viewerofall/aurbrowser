mod aur_client;
mod installer;
mod ui;
mod package_checker;

use gtk4::prelude::*;
use gtk4::Application;
use libadwaita as adw;

const APP_ID: &str = "com.aurbrowser.App";

fn main() {
    // engine startup
 let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
     let _guard = rt.enter();

    adw::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        let window = ui::window::build_ui(app);
        window.present();
    });

    app.run();
}
