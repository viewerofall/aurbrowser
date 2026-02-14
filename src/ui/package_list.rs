use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, ListBox, Orientation, SelectionMode, CheckButton};
use crate::aur_client::AurPackage;
use std::collections::HashSet;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_package_list() -> ListBox {
    let list_box = ListBox::new();
    list_box.set_selection_mode(SelectionMode::None);
    list_box.set_margin_top(10);
    list_box.set_margin_bottom(10);
    list_box.set_margin_start(10);
    list_box.set_margin_end(10);
    list_box
}

pub fn update_package_list(
    list_box: &ListBox,
    packages: Vec<AurPackage>,
    installed: &Rc<HashSet<String>>,
    bookmarks: &Rc<RefCell<Vec<String>>>,
) {
    // Clear existing items
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    // Add new packages
    for package in packages {
        let row = create_package_row(package, installed, bookmarks);
        list_box.append(&row);
    }
}

fn create_package_row(
    package: AurPackage,
    installed: &Rc<HashSet<String>>,
    bookmarks: &Rc<RefCell<Vec<String>>>,
) -> GtkBox {
    let row_box = GtkBox::new(Orientation::Horizontal, 10);
    row_box.set_margin_top(5);
    row_box.set_margin_bottom(5);
    row_box.set_margin_start(5);
    row_box.set_margin_end(5);

    let is_installed = installed.contains(&package.name);
    let is_bookmarked = bookmarks.borrow().contains(&package.name);

    // Bookmark checkbox
    let bookmark_check = CheckButton::new();
    bookmark_check.set_active(is_bookmarked);
    bookmark_check.set_tooltip_text(Some("Bookmark this package"));
    
    let pkg_name_for_bookmark = package.name.clone();
    let bookmarks_clone = bookmarks.clone();
    bookmark_check.connect_toggled(move |check| {
        let mut bookmarks = bookmarks_clone.borrow_mut();
        if check.is_active() {
            if !bookmarks.contains(&pkg_name_for_bookmark) {
                bookmarks.push(pkg_name_for_bookmark.clone());
            }
        } else {
            bookmarks.retain(|x| x != &pkg_name_for_bookmark);
        }
    });
    
    row_box.append(&bookmark_check);

    // Package info
    let info_box = GtkBox::new(Orientation::Vertical, 5);
    info_box.set_hexpand(true);

    let name_label = Label::new(Some(&format!(
        "{} {} {}",
        package.name,
        package.version,
        if is_installed { "✓ Installed" } else { "" }
    )));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("title-3");
    if is_installed {
        name_label.add_css_class("success");
    }

    let desc_label = Label::new(Some(
        &package.description.clone().unwrap_or_else(|| "No description".to_string())
    ));
    desc_label.set_halign(gtk4::Align::Start);
    desc_label.set_wrap(true);
    desc_label.set_xalign(0.0);

    let meta_text = format!(
        "Votes: {} | Popularity: {:.2} | Maintainer: {}",
        package.votes.unwrap_or(0),
        package.popularity.unwrap_or(0.0),
        package.maintainer.clone().unwrap_or_else(|| "None".to_string())
    );
    let meta_label = Label::new(Some(&meta_text));
    meta_label.set_halign(gtk4::Align::Start);
    meta_label.add_css_class("dim-label");

    info_box.append(&name_label);
    info_box.append(&desc_label);
    info_box.append(&meta_label);

    row_box.append(&info_box);

    // Button box
    let button_box = GtkBox::new(Orientation::Horizontal, 5);

    // Details button
    let details_button = Button::with_label("Details");
    let pkg_for_details = package.clone();
    details_button.connect_clicked(move |btn| {
        show_package_details(btn, &pkg_for_details);
    });
    button_box.append(&details_button);

    // Install button
    let install_button = Button::with_label(if is_installed { "Reinstall" } else { "Install" });
    if !is_installed {
        install_button.add_css_class("suggested-action");
    }

    let pkg_name = package.name.clone();
    install_button.connect_clicked(move |btn| {
        btn.set_label("Opening terminal...");
        btn.set_sensitive(false);
        
        if let Err(e) = crate::installer::spawn_yay_install(&pkg_name) {
            eprintln!("Failed to start installation: {}", e);
        }
        
        let btn_clone = btn.clone();
        glib::timeout_add_seconds_local(2, move || {
            btn_clone.set_label("Install");
            btn_clone.set_sensitive(true);
            glib::ControlFlow::Break
        });
    });
    button_box.append(&install_button);

    row_box.append(&button_box);

    row_box
}

fn show_package_details(parent_widget: &Button, package: &AurPackage) {
    let dialog = gtk4::Window::builder()
        .title(format!("Package Details: {}", package.name))
        .modal(true)
        .default_width(600)
        .default_height(500)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 10);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Header
    let header_label = Label::new(Some(&format!("{} {}", package.name, package.version)));
    header_label.add_css_class("title-1");
    header_label.set_halign(gtk4::Align::Start);
    main_box.append(&header_label);

    if let Some(desc) = &package.description {
        let desc_label = Label::new(Some(desc));
        desc_label.set_wrap(true);
        desc_label.set_xalign(0.0);
        desc_label.set_margin_bottom(10);
        main_box.append(&desc_label);
    }

    // Scrolled area for details
    let scrolled = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .build();

    let details_box = GtkBox::new(Orientation::Vertical, 10);

    // AUR Page Link
    let aur_link = format!("https://aur.archlinux.org/packages/{}", package.name);
    let link_button = gtk4::LinkButton::with_label(&aur_link, "View on AUR");
    link_button.set_halign(gtk4::Align::Start);
    details_box.append(&link_button);

    // Maintainer
    if let Some(maintainer) = &package.maintainer {
        add_detail_row(&details_box, "Maintainer:", maintainer);
    }

    // License
    if let Some(licenses) = &package.license {
        add_detail_row(&details_box, "License:", &licenses.join(", "));
    }

    // Votes and Popularity
    add_detail_row(&details_box, "Votes:", &package.votes.unwrap_or(0).to_string());
    add_detail_row(&details_box, "Popularity:", &format!("{:.2}", package.popularity.unwrap_or(0.0)));

    // Dependencies
    if let Some(deps) = &package.depends {
        if !deps.is_empty() {
            add_detail_section(&details_box, "Dependencies:", deps);
        }
    }

    // Make Dependencies
    if let Some(makedeps) = &package.makedepends {
        if !makedeps.is_empty() {
            add_detail_section(&details_box, "Make Dependencies:", makedeps);
        }
    }

    // Optional Dependencies
    if let Some(optdeps) = &package.optdepends {
        if !optdeps.is_empty() {
            add_detail_section(&details_box, "Optional Dependencies:", optdeps);
        }
    }

    // Conflicts
    if let Some(conflicts) = &package.conflicts {
        if !conflicts.is_empty() {
            add_detail_section(&details_box, "Conflicts:", conflicts);
        }
    }

    scrolled.set_child(Some(&details_box));
    main_box.append(&scrolled);

    // Close button
    let close_btn = Button::with_label("Close");
    close_btn.add_css_class("suggested-action");
    close_btn.set_halign(gtk4::Align::End);
    let dialog_clone = dialog.clone();
    close_btn.connect_clicked(move |_| {
        dialog_clone.close();
    });
    main_box.append(&close_btn);

    dialog.set_child(Some(&main_box));
    dialog.present();
}

fn add_detail_row(container: &GtkBox, label: &str, value: &str) {
    let row = GtkBox::new(Orientation::Horizontal, 10);
    
    let label_widget = Label::new(Some(label));
    label_widget.add_css_class("heading");
    label_widget.set_halign(gtk4::Align::Start);
    label_widget.set_width_request(180);
    
    let value_widget = Label::new(Some(value));
    value_widget.set_halign(gtk4::Align::Start);
    value_widget.set_wrap(true);
    value_widget.set_xalign(0.0);
    value_widget.set_hexpand(true);
    
    row.append(&label_widget);
    row.append(&value_widget);
    container.append(&row);
}

fn add_detail_section(container: &GtkBox, title: &str, items: &[String]) {
    let title_label = Label::new(Some(title));
    title_label.add_css_class("heading");
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_margin_top(10);
    container.append(&title_label);
    
    for item in items {
        let item_label = Label::new(Some(&format!("  • {}", item)));
        item_label.set_halign(gtk4::Align::Start);
        item_label.set_wrap(true);
        item_label.set_xalign(0.0);
        container.append(&item_label);
    }
}
