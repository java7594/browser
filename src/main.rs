use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Orientation};
use webkit2gtk::{WebView, WebViewExt, SettingsExt};

const APP_ID: &str = "org.browser";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("browser")
        .default_width(1024)
        .default_height(768)
        .build();

    let vbox = Box::new(Orientation::Vertical, 5);
    vbox.set_margin_top(5);
    vbox.set_margin_bottom(5);
    vbox.set_margin_start(5);
    vbox.set_margin_end(5);

    let hbox = Box::new(Orientation::Horizontal, 5);

    let back_btn = Button::with_label("←");
    hbox.pack_start(&back_btn, false, false, 0);

    let forward_btn = Button::with_label("→");
    hbox.pack_start(&forward_btn, false, false, 0);

    let refresh_btn = Button::with_label("⟳");
    hbox.pack_start(&refresh_btn, false, false, 0);

    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("Search with Ecosia or enter URL..."));
    hbox.pack_start(&url_entry, true, true, 0);

    let go_btn = Button::with_label("Go");
    hbox.pack_start(&go_btn, false, false, 0);

    vbox.pack_start(&hbox, false, false, 0);

    let webview = WebView::new();
    let settings = WebViewExt::settings(&webview).unwrap();
    settings.set_hardware_acceleration_policy(webkit2gtk::HardwareAccelerationPolicy::Always);
    vbox.pack_start(&webview, true, true, 0);

    webview.load_uri("https://ecosia.org");

    let wv = webview.clone();
    back_btn.connect_clicked(move |_| {
        wv.go_back();
    });

    let wv = webview.clone();
    forward_btn.connect_clicked(move |_| {
        wv.go_forward();
    });

    let wv = webview.clone();
    refresh_btn.connect_clicked(move |_| {
        wv.reload();
    });

    let wv = webview.clone();
    let entry = url_entry.clone();
    go_btn.connect_clicked(move |_| {
        let text = entry.text().to_string();
        load_url(&wv, &text);
    });

    let wv = webview.clone();
    url_entry.connect_activate(move |entry| {
        let text = entry.text().to_string();
        load_url(&wv, &text);
    });

    webview.connect_load_changed(move |wv, _| {
        if let Some(uri) = wv.uri() {
            url_entry.set_text(&uri);
        }
    });

    window.add(&vbox);
    window.show_all();
}

fn load_url(webview: &WebView, input: &str) {
    let url = if input.starts_with("http://") || input.starts_with("https://") {
        input.to_string()
    } else if input.contains('.') && !input.contains(' ') {
        format!("https://{}", input)
    } else {
        format!("https://ecosia.org/search?q={}", urlencoding::encode(input))
    };
    
    webview.load_uri(&url);
}