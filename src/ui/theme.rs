use dioxus::prelude::*;

/// Global signal holding the current theme class ("dark" or "").
pub static THEME_CLASS: GlobalSignal<&'static str> = Signal::global(|| "");

pub fn get_theme_class(theme_setting: &str) -> &'static str {
    match theme_setting {
        "dark" => "dark",
        _ => "",
    }
}
