use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Box, Button, Entry, Orientation, Notebook, ComboBoxText, Label};
use gtk::glib;
use webkit2gtk::{WebView, WebViewExt, SettingsExt, WebContext, WebContextExt, WebsiteDataManagerExt, WebsiteDataManagerExtManual, CookieManagerExt};
use gtk::gio;
use std::rc::Rc;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;

const APP_ID: &str = "org.browser";

struct SearchEngine {
    name: &'static str,
    url: &'static str,
}

const SEARCH_ENGINES: &[SearchEngine] = &[
    SearchEngine { name: "Ecosia", url: "https://ecosia.org/search?q=" },
    SearchEngine { name: "DuckDuckGo", url: "https://duckduckgo.com/?q=" },
    SearchEngine { name: "Brave", url: "https://search.brave.com/search?q=" },
    SearchEngine { name: "Google", url: "https://www.google.com/search?q=" },
    SearchEngine { name: "Bing", url: "https://www.bing.com/search?q=" },
    SearchEngine { name: "Kagi", url: "https://kagi.com/search?q=" },
    SearchEngine { name: "Yahoo", url: "https://search.yahoo.com/search?q=" },
];

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

fn get_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("browser");
    fs::create_dir_all(&path).ok();
    path
}

fn get_settings_path() -> PathBuf {
    let mut path = get_config_path();
    path.push("settings.txt");
    path
}

fn get_data_path() -> PathBuf {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("browser");
    fs::create_dir_all(&path).ok();
    path
}

fn load_search_engine() -> usize {
    let path = get_settings_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_search_engine(index: usize) {
    let path = get_settings_path();
    fs::write(path, index.to_string()).ok();
}

fn build_ui(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("browser")
        .default_width(1024)
        .default_height(768)
        .build();
    
    window.maximize();

    let vbox = Box::new(Orientation::Vertical, 5);
    vbox.set_margin_top(5);
    vbox.set_margin_bottom(5);
    vbox.set_margin_start(5);
    vbox.set_margin_end(5);

    let notebook = Notebook::new();
    notebook.set_scrollable(true);
    
    let web_context = WebContext::default().unwrap();
    let data_manager = web_context.website_data_manager().unwrap();
    let data_path = get_data_path();
    let cookie_manager = data_manager.cookie_manager().unwrap();
    
    let mut cookie_path = data_path.clone();
    cookie_path.push("cookies.txt");
    cookie_manager.set_persistent_storage(&cookie_path.to_string_lossy(), webkit2gtk::CookiePersistentStorage::Text);
    
    let web_context_rc = Rc::new(web_context);
    let notebook_rc = Rc::new(RefCell::new(notebook.clone()));
    let search_engine_index = Rc::new(RefCell::new(load_search_engine()));

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
    url_entry.set_placeholder_text(Some("Search or enter URL..."));
    hbox.pack_start(&url_entry, true, true, 0);

    let go_btn = Button::with_label("Go");
    hbox.pack_start(&go_btn, false, false, 0);

    let settings_btn = Button::with_label("⚙");
    hbox.pack_start(&settings_btn, false, false, 0);

    vbox.pack_start(&hbox, false, false, 0);
    vbox.pack_start(&notebook, true, true, 0);

    let nb = notebook_rc.clone();
    let entry_new_tab = url_entry.clone();
    let se_idx = search_engine_index.clone();
    let wc = web_context_rc.clone();
    new_tab_btn.connect_clicked(move |_| {
        let idx = *se_idx.borrow();
        let home_url = get_search_engine_homepage(idx);
        create_tab(&nb.borrow(), &home_url, &entry_new_tab, &wc);
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
    let se_idx = search_engine_index.clone();
    go_btn.connect_clicked(move |_| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            let text = entry.text().to_string();
            let idx = *se_idx.borrow();
            load_url(&wv, &text, idx);
        }
    });

    let nb = notebook_rc.clone();
    let se_idx = search_engine_index.clone();
    url_entry.connect_activate(move |entry| {
        if let Some(wv) = get_current_webview(&nb.borrow()) {
            let text = entry.text().to_string();
            let idx = *se_idx.borrow();
            load_url(&wv, &text, idx);
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

    let nb = notebook_rc.clone();
    let se_idx = search_engine_index.clone();
    let wc = web_context_rc.clone();
    settings_btn.connect_clicked(move |_| {
        let settings_page = create_settings_tab(&nb.borrow(), se_idx.clone(), &wc);
        nb.borrow().set_current_page(Some(settings_page));
    });

    let initial_idx = *search_engine_index.borrow();
    let home_url = get_search_engine_homepage(initial_idx);
    create_tab(&notebook, &home_url, &url_entry, &web_context_rc);

    window.add(&vbox);
    window.show_all();
}

fn create_settings_tab(notebook: &Notebook, search_engine_index: Rc<RefCell<usize>>, web_context: &Rc<WebContext>) -> u32 {
    let settings_box = Box::new(Orientation::Vertical, 20);
    settings_box.set_margin_top(20);
    settings_box.set_margin_bottom(20);
    settings_box.set_margin_start(20);
    settings_box.set_margin_end(20);

    let title = Label::new(Some("Settings"));
    title.set_markup("<span size='xx-large' weight='bold'>Settings</span>");
    settings_box.pack_start(&title, false, false, 0);

    let search_section = Box::new(Orientation::Vertical, 10);
    let search_label = Label::new(Some("Search Engine:"));
    search_label.set_halign(gtk::Align::Start);
    search_section.pack_start(&search_label, false, false, 0);

    let search_combo = ComboBoxText::new();
    for engine in SEARCH_ENGINES {
        search_combo.append_text(engine.name);
    }
    search_combo.set_active(Some(*search_engine_index.borrow() as u32));
    search_section.pack_start(&search_combo, false, false, 0);

    settings_box.pack_start(&search_section, false, false, 0);

    let cookie_section = Box::new(Orientation::Vertical, 10);
    let cookie_label = Label::new(Some("Privacy:"));
    cookie_label.set_halign(gtk::Align::Start);
    cookie_section.pack_start(&cookie_label, false, false, 0);

    let clear_cookies_btn = Button::with_label("Clear All Cookies");
    cookie_section.pack_start(&clear_cookies_btn, false, false, 0);
    cookie_section.show_all();

    settings_box.pack_start(&cookie_section, false, false, 0);

    let wc = web_context.clone();
    clear_cookies_btn.connect_clicked(move |_| {
        if let Some(data_manager) = wc.website_data_manager() {
            if let Some(cookie_manager) = data_manager.cookie_manager() {
                let mut cookie_path = get_data_path();
                cookie_path.push("cookies.txt");
                
                // Delete the cookie file
                std::fs::remove_file(&cookie_path).ok();
                
                // Recreate empty cookie file
                std::fs::File::create(&cookie_path).ok();
                
                // Set persistent storage again
                cookie_manager.set_persistent_storage(&cookie_path.to_string_lossy(), webkit2gtk::CookiePersistentStorage::Text);
            }
        }
    });

    let se_idx = search_engine_index.clone();
    search_combo.connect_changed(move |combo| {
        if let Some(idx) = combo.active() {
            let index = idx as usize;
            *se_idx.borrow_mut() = index;
            save_search_engine(index);
        }
    });

    let tab_box = Box::new(Orientation::Horizontal, 5);
    let tab_label = Label::new(Some("Settings"));
    let close_btn = Button::with_label("×");
    close_btn.set_relief(gtk::ReliefStyle::None);
    
    tab_box.pack_start(&tab_label, true, true, 0);
    tab_box.pack_start(&close_btn, false, false, 0);
    tab_box.show_all();

    let page_num = notebook.append_page(&settings_box, Some(&tab_box));
    
    let nb = notebook.clone();
    let sb = settings_box.clone();
    close_btn.connect_clicked(move |_| {
        if let Some(page) = nb.page_num(&sb) {
            nb.remove_page(Some(page));
        }
    });
    
    settings_box.show_all();
    page_num
}

fn create_tab(notebook: &Notebook, url: &str, url_entry: &Entry, web_context: &Rc<WebContext>) {
    let webview = WebView::with_context(web_context.as_ref());
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
    let entry_close = url_entry.clone();
    close_btn.connect_clicked(move |_| {
        if let Some(page) = nb.page_num(&wv) {
            let total_pages = nb.n_pages();
            nb.remove_page(Some(page));
            
            if total_pages > 1 {
                if let Some(current_wv) = get_current_webview(&nb) {
                    if let Some(uri) = current_wv.uri() {
                        entry_close.set_text(&uri);
                    }
                }
            }
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

fn load_url(webview: &WebView, input: &str, search_engine_idx: usize) {
    let url = if input.starts_with("http://") || input.starts_with("https://") {
        input.to_string()
    } else if input.contains('.') && !input.contains(' ') {
        format!("https://{}", input)
    } else {
        format!("{}{}", SEARCH_ENGINES[search_engine_idx].url, urlencoding::encode(input))
    };
    
    webview.load_uri(&url);
}

fn get_search_engine_homepage(index: usize) -> String {
    format!("https://{}", SEARCH_ENGINES[index].url.split("://").nth(1).unwrap().split('/').next().unwrap())
}
