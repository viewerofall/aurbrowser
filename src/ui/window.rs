use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, ScrolledWindow, DropDown, Spinner, Button};
use libadwaita as adw;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq)]
pub enum SortMode {
    Popularity,
    Votes,
    Alphabetical,
    LastModified,
}

pub fn build_ui(app: &Application) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("AUR Browser")
        .default_width(1000)
        .default_height(800)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Header bar with favorite/bookmark button
    let header = adw::HeaderBar::new();
    let bookmarks_btn = Button::with_label("★ Bookmarks");
    header.pack_start(&bookmarks_btn);
    main_box.append(&header);

    // Search and sort controls
    let controls_box = GtkBox::new(Orientation::Horizontal, 10);
    controls_box.set_margin_top(10);
    controls_box.set_margin_bottom(10);
    controls_box.set_margin_start(10);
    controls_box.set_margin_end(10);

    // Search bar
    let search_entry = gtk4::Entry::builder()
        .placeholder_text("Search AUR packages... (Press Enter)")
        .hexpand(true)
        .build();
    controls_box.append(&search_entry);

    // Sort dropdown
    let sort_label = gtk4::Label::new(Some("Sort:"));
    controls_box.append(&sort_label);

    let sort_options = gtk4::StringList::new(&[
        "Popularity",
        "Votes", 
        "Alphabetical",
        "Last Modified",
    ]);
    let sort_dropdown = DropDown::new(Some(sort_options), None::<gtk4::Expression>);
    sort_dropdown.set_selected(0);
    controls_box.append(&sort_dropdown);

    main_box.append(&controls_box);

    // Content area (will contain either package list, loading spinner, or error)
    let content_stack = gtk4::Stack::new();
    content_stack.set_vexpand(true);

    // Package list view
    let scrolled_window = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .build();

    let package_list_box = super::package_list::create_package_list();
    scrolled_window.set_child(Some(&package_list_box));
    content_stack.add_named(&scrolled_window, Some("packages"));

    // Loading view
    let loading_box = GtkBox::new(Orientation::Vertical, 20);
    loading_box.set_valign(gtk4::Align::Center);
    loading_box.set_halign(gtk4::Align::Center);
    let spinner = Spinner::new();
    spinner.set_size_request(48, 48);
    spinner.start();
    let loading_label = gtk4::Label::new(Some("Loading packages..."));
    loading_label.add_css_class("title-2");
    loading_box.append(&spinner);
    loading_box.append(&loading_label);
    content_stack.add_named(&loading_box, Some("loading"));

    // Error view
    let error_box = GtkBox::new(Orientation::Vertical, 20);
    error_box.set_valign(gtk4::Align::Center);
    error_box.set_halign(gtk4::Align::Center);
    let error_icon = gtk4::Label::new(Some("⚠️"));
    error_icon.add_css_class("title-1");
    let error_label = gtk4::Label::new(Some("Failed to connect to AUR"));
    error_label.add_css_class("title-2");
    let error_detail = gtk4::Label::new(Some("Check your internet connection"));
    error_detail.add_css_class("dim-label");
    let retry_button = Button::with_label("Retry");
    retry_button.add_css_class("suggested-action");
    retry_button.set_size_request(120, -1);
    error_box.append(&error_icon);
    error_box.append(&error_label);
    error_box.append(&error_detail);
    error_box.append(&retry_button);
    content_stack.add_named(&error_box, Some("error"));

    // Empty state view
    let empty_box = GtkBox::new(Orientation::Vertical, 20);
    empty_box.set_valign(gtk4::Align::Center);
    empty_box.set_halign(gtk4::Align::Center);
    let empty_icon = gtk4::Label::new(Some("🔍"));
    empty_icon.add_css_class("title-1");
    let empty_label = gtk4::Label::new(Some("No packages found"));
    empty_label.add_css_class("title-2");
    let empty_detail = gtk4::Label::new(Some("Try a different search term"));
    empty_detail.add_css_class("dim-label");
    empty_box.append(&empty_icon);
    empty_box.append(&empty_label);
    empty_box.append(&empty_detail);
    content_stack.add_named(&empty_box, Some("empty"));

    main_box.append(&content_stack);
    window.set_child(Some(&main_box));

    // Store current packages and bookmarks
    let current_packages: Rc<RefCell<Vec<crate::aur_client::AurPackage>>> = Rc::new(RefCell::new(Vec::new()));
    let bookmarked_packages: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let installed_packages = Rc::new(crate::package_checker::get_installed_packages());

    // Load initial packages
    content_stack.set_visible_child_name("loading");
    let package_list_clone = package_list_box.clone();
    let current_packages_clone = current_packages.clone();
    let content_stack_clone = content_stack.clone();
    let installed_clone = installed_packages.clone();
    let bookmarks_clone = bookmarked_packages.clone();
    glib::MainContext::default().spawn_local(async move {
        match crate::aur_client::get_recent_packages(50).await {
            Ok(mut packages) => {
                packages.sort_by(|a, b| {
                    b.popularity.unwrap_or(0.0).partial_cmp(&a.popularity.unwrap_or(0.0)).unwrap()
                });
                *current_packages_clone.borrow_mut() = packages.clone();
                super::package_list::update_package_list(
                    &package_list_clone,
                    packages,
                    &installed_clone,
                    &bookmarks_clone,
                );
                content_stack_clone.set_visible_child_name("packages");
            }
            Err(e) => {
                eprintln!("Failed to load packages: {}", e);
                content_stack_clone.set_visible_child_name("error");
            }
        }
    });

    // Retry button handler
    let package_list_clone = package_list_box.clone();
    let current_packages_clone = current_packages.clone();
    let content_stack_clone = content_stack.clone();
    let installed_clone = installed_packages.clone();
    let bookmarks_clone = bookmarked_packages.clone();
    retry_button.connect_clicked(move |_| {
        content_stack_clone.set_visible_child_name("loading");
        let package_list = package_list_clone.clone();
        let current_packages = current_packages_clone.clone();
        let content_stack = content_stack_clone.clone();
        let installed = installed_clone.clone();
        let bookmarks = bookmarks_clone.clone();
        
        glib::MainContext::default().spawn_local(async move {
            match crate::aur_client::get_recent_packages(50).await {
                Ok(mut packages) => {
                    packages.sort_by(|a, b| {
                        b.popularity.unwrap_or(0.0).partial_cmp(&a.popularity.unwrap_or(0.0)).unwrap()
                    });
                    *current_packages.borrow_mut() = packages.clone();
                    super::package_list::update_package_list(&package_list, packages, &installed, &bookmarks);
                    content_stack.set_visible_child_name("packages");
                }
                Err(e) => {
                    eprintln!("Failed to load packages: {}", e);
                    content_stack.set_visible_child_name("error");
                }
            }
        });
    });

    // Search functionality
    let package_list_clone = package_list_box.clone();
    let current_packages_clone = current_packages.clone();
    let sort_dropdown_clone = sort_dropdown.clone();
    let content_stack_clone = content_stack.clone();
    let installed_clone = installed_packages.clone();
    let bookmarks_clone = bookmarked_packages.clone();
    search_entry.connect_activate(move |entry| {
        let query = entry.text().to_string();
        if query.is_empty() {
            return;
        }

        content_stack_clone.set_visible_child_name("loading");
        let package_list = package_list_clone.clone();
        let current_packages = current_packages_clone.clone();
        let sort_dropdown = sort_dropdown_clone.clone();
        let content_stack = content_stack_clone.clone();
        let installed = installed_clone.clone();
        let bookmarks = bookmarks_clone.clone();
        
        glib::MainContext::default().spawn_local(async move {
            match crate::aur_client::search_aur(&query).await {
                Ok(mut packages) => {
                    if packages.is_empty() {
                        content_stack.set_visible_child_name("empty");
                        return;
                    }
                    
                    let sort_mode = match sort_dropdown.selected() {
                        0 => SortMode::Popularity,
                        1 => SortMode::Votes,
                        2 => SortMode::Alphabetical,
                        3 => SortMode::LastModified,
                        _ => SortMode::Popularity,
                    };
                    sort_packages(&mut packages, sort_mode);
                    *current_packages.borrow_mut() = packages.clone();
                    super::package_list::update_package_list(&package_list, packages, &installed, &bookmarks);
                    content_stack.set_visible_child_name("packages");
                }
                Err(e) => {
                    eprintln!("Search failed: {}", e);
                    content_stack.set_visible_child_name("error");
                }
            }
        });
    });

    // Sort dropdown handler
    let package_list_clone = package_list_box.clone();
    let current_packages_clone = current_packages.clone();
    let installed_clone = installed_packages.clone();
    let bookmarks_clone = bookmarked_packages.clone();
    sort_dropdown.connect_selected_notify(move |dropdown| {
        let sort_mode = match dropdown.selected() {
            0 => SortMode::Popularity,
            1 => SortMode::Votes,
            2 => SortMode::Alphabetical,
            3 => SortMode::LastModified,
            _ => SortMode::Popularity,
        };
        
        let mut packages = current_packages_clone.borrow().clone();
        if !packages.is_empty() {
            sort_packages(&mut packages, sort_mode);
            super::package_list::update_package_list(
                &package_list_clone,
                packages,
                &installed_clone,
                &bookmarks_clone,
            );
        }
    });

    // Bookmarks button handler
    let package_list_clone = package_list_box.clone();
    let bookmarked_clone = bookmarked_packages.clone();
    let current_packages_clone = current_packages.clone();
    let content_stack_clone = content_stack.clone();
    let installed_clone = installed_packages.clone();
    bookmarks_btn.connect_clicked(move |_| {
        let bookmarked = bookmarked_clone.borrow();
        let all_packages = current_packages_clone.borrow();
        
        let bookmarked_list: Vec<_> = all_packages
            .iter()
            .filter(|p| bookmarked.contains(&p.name))
            .cloned()
            .collect();
        
        if bookmarked_list.is_empty() {
            content_stack_clone.set_visible_child_name("empty");
        } else {
            super::package_list::update_package_list(
                &package_list_clone,
                bookmarked_list,
                &installed_clone,
                &bookmarked_clone,
            );
            content_stack_clone.set_visible_child_name("packages");
        }
    });

    window
}

fn sort_packages(packages: &mut Vec<crate::aur_client::AurPackage>, mode: SortMode) {
    match mode {
        SortMode::Popularity => {
            packages.sort_by(|a, b| {
                b.popularity.unwrap_or(0.0).partial_cmp(&a.popularity.unwrap_or(0.0)).unwrap()
            });
        }
        SortMode::Votes => {
            packages.sort_by(|a, b| {
                b.votes.unwrap_or(0).cmp(&a.votes.unwrap_or(0))
            });
        }
        SortMode::Alphabetical => {
            packages.sort_by(|a, b| a.name.cmp(&b.name));
        }
        SortMode::LastModified => {
            packages.sort_by(|a, b| {
                b.last_modified.unwrap_or(0).cmp(&a.last_modified.unwrap_or(0))
            });
        }
    }
}
