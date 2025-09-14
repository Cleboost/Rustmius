use gtk4::prelude::*;
use gtk4::{MenuButton, PopoverMenu, gio::Menu, gio::SimpleAction};
use libadwaita::prelude::*;
use libadwaita::{
    Application, ApplicationWindow, HeaderBar, StyleManager, ToastOverlay, ViewStack, ViewSwitcher,
};
use std::rc::Rc;

mod service;
mod ui;
use crate::service::load_settings;
use crate::ui::bus::set_history_refresh;
use ui::modal::about::create_about_dialog;
use ui::tab::{create_history_tab, create_key_tab, create_server_tab};

use crate::ui::modal::preference::create_preference_dialog;

fn create_point_menu_model(
    window: &ApplicationWindow,
    on_settings_changed: std::rc::Rc<dyn Fn()>,
) -> Menu {
    let point_menu = Menu::new();

    point_menu.append(Some("Paramètres"), Some("app.settings"));
    point_menu.append(Some("À propos"), Some("app.about"));
    point_menu.append(Some("Aide"), Some("app.help"));
    point_menu.append(Some("Quitter"), Some("app.quit"));

    let settings_action = SimpleAction::new("settings", None);
    let window_clone = window.clone();
    settings_action.connect_activate(move |_, _| {
        let settings_dialog = create_preference_dialog(Some(on_settings_changed.clone()));
        settings_dialog.present(Some(&window_clone));
    });

    let about_action = SimpleAction::new("about", None);
    let window_clone = window.clone();
    about_action.connect_activate(move |_, _| {
        let about_dialog = create_about_dialog();
        about_dialog.present(Some(&window_clone));
    });

    let help_action = SimpleAction::new("help", None);
    help_action.connect_activate(move |_, _| {
        println!("Aide - Fonctionnalité à implémenter");
    });

    let quit_action = SimpleAction::new("quit", None);
    let window_clone2 = window.clone();
    quit_action.connect_activate(move |_, _| {
        window_clone2.close();
    });

    if let Some(app) = window.application() {
        app.add_action(&settings_action);
        app.add_action(&about_action);
        app.add_action(&help_action);
        app.add_action(&quit_action);
    }

    point_menu
}

fn main() {
    libadwaita::init().expect("Failed to initialize libadwaita");

    let app = Application::builder()
        .application_id("com.com.rustmius")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let style_manager = StyleManager::default();
    style_manager.set_color_scheme(libadwaita::ColorScheme::Default);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Rustmius")
        .default_width(800)
        .default_height(600)
        .build();

    let toast_overlay = Rc::new(ToastOverlay::new());

    let stack = ViewStack::new();

    let (key_tab, _key_refresh_fn) = create_key_tab(Rc::clone(&toast_overlay), Some(&window));
    let (server_tab, _server_refresh_fn) = create_server_tab(Some(&window));

    let server_page = stack.add_titled(&server_tab, Some("server"), "Serveurs");
    let key_page = stack.add_titled(&key_tab, Some("key"), "Clés SSH");

    let settings = load_settings();
    if settings.remember_servers {
        let (history_tab, history_refresh) = create_history_tab();
        let history_page = stack.add_titled(&history_tab, Some("history"), "History");
        history_page.set_icon_name(Some("document-open-recent-symbolic"));
        set_history_refresh(Some(history_refresh));
    } else {
        set_history_refresh(None);
    }

    key_page.set_icon_name(Some("encryption-symbolic"));
    server_page.set_icon_name(Some("network-server"));

    let view_switcher = ViewSwitcher::new();
    view_switcher.set_stack(Some(&stack));
    view_switcher.set_policy(libadwaita::ViewSwitcherPolicy::Wide);

    let header = HeaderBar::new();
    header.set_title_widget(Some(&view_switcher.clone()));

    let stack_for_cb = stack.clone();
    let on_settings_changed_cb: Rc<dyn Fn()> = Rc::new(move || {
        let settings = load_settings();

        let pages = stack_for_cb.pages();
        let mut history_child: Option<gtk4::Widget> = None;
        for i in 0..pages.n_items() {
            if let Some(obj) = pages.item(i) {
                if let Ok(page) = obj.downcast::<libadwaita::ViewStackPage>() {
                    if page.name().as_deref() == Some("history") {
                        history_child = Some(page.child());
                        break;
                    }
                }
            }
        }

        if settings.remember_servers {
            if history_child.is_none() {
                let (history_tab, history_refresh) = create_history_tab();
                let history_page =
                    stack_for_cb.add_titled(&history_tab, Some("history"), "History");
                history_page.set_icon_name(Some("document-open-recent-symbolic"));
                set_history_refresh(Some(history_refresh));
            }
        } else if let Some(child) = history_child {
            stack_for_cb.remove(&child);
            set_history_refresh(None);
        }
    });

    let menu_button = MenuButton::new();
    menu_button.set_icon_name("open-menu-symbolic");
    menu_button.set_tooltip_text(Some("Menu principal"));

    let menu_model = create_point_menu_model(&window, Rc::clone(&on_settings_changed_cb));
    let menu = PopoverMenu::from_model(Some(&menu_model));
    menu_button.set_popover(Some(&menu));

    header.pack_end(&menu_button);

    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.append(&header);
    main_box.append(&stack);

    toast_overlay.set_child(Some(&main_box));

    window.set_content(Some(&*toast_overlay));
    window.present();
}
