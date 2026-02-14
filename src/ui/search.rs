use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Entry, Orientation};

pub fn create_search_bar() -> GtkBox {
    let search_box = GtkBox::new(Orientation::Horizontal, 10);
    search_box.set_margin_top(10);
    search_box.set_margin_bottom(10);
    search_box.set_margin_start(10);
    search_box.set_margin_end(10);

    let search_entry = Entry::builder()
        .placeholder_text("Search AUR packages... (Press Enter)")
        .hexpand(true)
        .build();

    search_box.append(&search_entry);
    search_box
}

pub fn connect_search_handler(
    search_box: &GtkBox,
    package_list: &gtk4::ListBox,
) {
    let search_entry = search_box
        .first_child()
        .and_then(|w| w.downcast::<gtk4::Entry>().ok())
        .expect("Search entry not found");

    let package_list_clone = package_list.clone();

    search_entry.connect_activate(move |entry| {
        let query = entry.text().to_string();
        if query.is_empty() {
            return;
        }

        let package_list = package_list_clone.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Ok(packages) = crate::aur_client::search_aur(&query).await {
                super::package_list::update_package_list(&package_list, packages);
            }
        });
    });
}
