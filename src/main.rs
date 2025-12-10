use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Orientation, Notebook};
use webkit2gtk::{WebView, WebViewExt, SettingsExt};
use std::rc::Rc;
use std::cell::RefCell;

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

    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    
    let notebook_rc = Rc::new(RefCell::new(notebook.clone()));

    let hbox = Box::new(Orientation::Horizontal, 5);

    let new_tab_btn = Button::with_label("+");
    hbox.pack_start(&new_tab_btn, false, false, 0);

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
    vbox.pack_start(&notebook, true, true, 0);

    let nb = notebook_rc.clone();
    let entry_new_tab = url_entry.clone();
    new_tab_btn.connect_clicked(move |_| {
        create_tab(&nb.borrow(), "https://ecosia.org", &entry_new_tab);
    });

    let nb = notebook_rc.clone();
    back_btn.connect_clicked(move |_| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            wv.go_back();
        }
    });

    let nb = notebook_rc.clone();
    forward_btn.connect_clicked(move |_| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            wv.go_forward();
        }
    });

    let nb = notebook_rc.clone();
    refresh_btn.connect_clicked(move |_| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            wv.reload();
        }
    });

    let nb = notebook_rc.clone();
    let entry = url_entry.clone();
    go_btn.connect_clicked(move |_| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            let text = entry.text().to_string();
            load_url(&wv, &text);
        }
    });

    let nb = notebook_rc.clone();
    url_entry.connect_activate(move |entry| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            let text = entry.text().to_string();
            load_url(&wv, &text);
        }
    });

    let entry_clone = url_entry.clone();
    let nb_clone = notebook.clone();
    notebook.connect_switch_page(move |_, _, page_num| {
        if let Some(widget) = nb_clone.nth_page(Some(page_num)) {
            if let Ok(wv) = widget.downcast::<WebView>() {
                if let Some(uri) = wv.uri() {
                    entry_clone.set_text(&uri);
                }
            }
        }
    });

    create_tab(&notebook, "https://ecosia.org", &url_entry);

    window.add(&vbox);
    window.show_all();
}

fn create_tab(notebook: &Notebook, url: &str, url_entry: &Entry) {
    let webview = WebView::new();
    let settings = WebViewExt::settings(&webview).unwrap();
    settings.set_hardware_acceleration_policy(webkit2gtk::HardwareAccelerationPolicy::Always);
    
    let tab_box = Box::new(Orientation::Horizontal, 5);
    let label = gtk::Label::new(Some("New Tab"));
    let close_btn = Button::with_label("×");
    close_btn.set_relief(gtk::ReliefStyle::None);
    
    tab_box.pack_start(&label, true, true, 0);
    tab_box.pack_start(&close_btn, false, false, 0);
    tab_box.show_all();

    let page_num = notebook.append_page(&webview, Some(&tab_box));
    notebook.set_current_page(Some(page_num));
    
    let nb = notebook.clone();
    let wv = webview.clone();
    close_btn.connect_clicked(move |_| {
        if let Some(page) = nb.page_num(&wv) {
            nb.remove_page(Some(page));
        }
    });

    let lbl = label.clone();
    let entry = url_entry.clone();
    let nb2 = notebook.clone();
    let wv2 = webview.clone();
    webview.connect_load_changed(move |wv, _| {
        if let Some(title) = wv.title() {
            lbl.set_text(&title);
        }
        if let Some(uri) = wv.uri() {
            if let Some(current) = get_current_webview(&nb2) {
                if current == wv2 {
                    entry.set_text(&uri);
                }
            }
        }
    });

    webview.load_uri(url);
    webview.show();
}

fn get_current_webview(notebook: &Notebook) -> Option<WebView> {
    notebook.current_page()
        .and_then(|page| notebook.nth_page(Some(page)))
        .and_then(|widget| widget.downcast::<WebView>().ok())
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
