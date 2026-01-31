use gtk::prelude::*;
use gdk_pixbuf::{Pixbuf, PixbufLoader};

pub fn get_svg(name: &str) -> Option<String> {
    let path = match name {
        "inbox" => r#"<path d="M22 12h-6l-2 3h-4l-2-3H2v12h20a2 2 0 0 0 2-2V12zm-20 0v-6h20v6"/>"#,
        // The search result for inbox was complex or from a different set. Let's use a simpler path or the one found.
        // Found: M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z
        // That's the "Inbox" icon.
        "inbox_alt" => r#"<path d="M22 12h-6l-2 3h-4l-2-3H2v12h20a2 2 0 0 0 2-2V12z" /><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />"#,
        "calendar" => r#"<path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h21"/>"#,
        "calendar-days" => r#"<path d="M8 2v4"/><path d="M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/><path d="M8 14h.01"/><path d="M12 14h.01"/><path d="M16 14h.01"/><path d="M8 18h.01"/><path d="M12 18h.01"/><path d="M16 18h.01"/>"#,
        "folder" => r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.98l-.81-1.2A2 2 0 0 0 11.1 2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h15z"/>"#,
        "plus" => r#"<path d="M5 12h14"/><path d="M12 5v14"/>"#,
        "settings" => r#"<path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.47a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/>"#,
        "trash-2" => r#"<path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><line x1="10" x2="10" y1="11" y2="17"/><line x1="14" x2="14" y1="11" y2="17"/>"#,
        "pencil" => r#"<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/><path d="m15 5 4 4"/>"#,
        "check" => r#"<path d="M20 6 9 17l-5-5"/>"#,
        "circle" => r#"<circle cx="12" cy="12" r="10"/>"#,
        _ => return None,
    };
    
    // We construct a valid SVG string
    // We use currentColor so it adapts, but PixbufLoader might not support CSS vars directly or easily without context.
    // However, for icons in buttons, we usually want them to match text color.
    // We'll set a default fill/stroke to black or allow it to be colored.
    // Actually, to support theming properly (dark/light), we should probably let GTK handle coloring if possible,
    // or return a raw SVG and use it.
    // For now, let's use a standard gray that works in both or black.
    // Better yet: hardcode "currentColor" and hope the renderer picks it up or defaults to black?
    // Librsvg (used by GdkPixbuf) supports currentColor if the context is set, but here we just load a pixbuf.
    // We will default to a dark gray #555555. If user wants specific, we might need to parameterize.
    
    // Use standard string concatenation to avoid any macro parsing ambiguity
    Some(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"#555555\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\">{}</svg>",
        path
    ))
}

pub fn get_icon_pixbuf(name: &str, size: i32) -> Option<Pixbuf> {
    if let Some(svg_data) = get_svg(name) {
        let loader = PixbufLoader::with_type("svg").ok()?;
        loader.write(svg_data.as_bytes()).ok()?;
        loader.close().ok()?;
        
        let pixbuf = loader.pixbuf()?;
        // Scale if needed, but SVG should scale naturally if we set size in load? 
        // PixbufLoader doesn't allow setting size before load easily.
        // We can scale the resulting pixbuf.
        
        let w = pixbuf.width();
        let h = pixbuf.height();
        if w != size || h != size {
            return pixbuf.scale_simple(size, size, gdk_pixbuf::InterpType::Bilinear);
        }
        return Some(pixbuf);
    }
    None
}
