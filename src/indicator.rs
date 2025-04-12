use std::rc::Rc;

use gtk4::{
    Application, ApplicationWindow,
    gdk::prelude::{DisplayExt, MonitorExt},
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    glib::object::Cast,
    prelude::{GtkApplicationExt, GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;

fn init_context() -> Context {
    let mut context = Context::default();

    // Get the display
    let display = gtk4::gdk::Display::default().expect("Could not get default display");

    // Get all monitors on the display
    let monitors = display.monitors();

    let mut monitor_list = vec![];

    for monitor in (&monitors).into_iter().flatten() {
        if let Ok(mon) = monitor.downcast::<gtk4::gdk::Monitor>() {
            monitor_list.push(mon);
        }
    }

    if monitor_list.is_empty() {
        panic!("No monitor found");
    }

    let monitor = monitor_list[0].clone();
    let geometry = monitor.geometry();
    context.offset_x = geometry.x();
    context.offset_y = geometry.y();

    context
}

#[derive(Default, Clone)]
struct Context {
    offset_x: i32,
    offset_y: i32,
}

pub fn start_gtk(
    mut rx: tokio::sync::mpsc::Receiver<(i32, i32, i32, i32)>,
) -> gtk4::glib::ExitCode {
    gtk4::init().expect("Cannot initialize gtk");
    let context = init_context();

    let app = Rc::new(
        Application::builder()
            .application_id("com.example.gtk-layer-shell-demo")
            .build(),
    );

    app.connect_activate(build_ui);

    let app_clone = app.clone();

    gtk4::glib::MainContext::default().spawn_local(async move {
        let context = context;

        while let Some((w, h, x, y)) = rx.recv().await {
            let window = app_clone
                .windows()
                .get(0)
                .expect("Cannot get window")
                .clone();

            window.set_margin(gtk4_layer_shell::Edge::Left, x - context.offset_x);
            window.set_margin(gtk4_layer_shell::Edge::Top, y - context.offset_y);

            window.set_width_request(w);
            window.set_height_request(h);

            window.set_visible(true);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            window.set_visible(false);
        }
    });

    app.run()
}

fn build_ui(app: &Application) {
    // Create a window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Layer Shell Demo")
        .build();

    window.init_layer_shell();
    window.set_namespace(Some("auto_fishing_indicator"));
    window.set_layer(gtk4_layer_shell::Layer::Overlay);

    window.set_anchor(gtk4_layer_shell::Edge::Left, true);
    window.set_anchor(gtk4_layer_shell::Edge::Top, true);

    // Show the window
    window.present();
    window.set_visible(false);
}
