use crate::model::Lang;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

const FALLBACK: &str = "en";

type Table = HashMap<String, HashMap<String, String>>;

static TABLE: OnceLock<Table> = OnceLock::new();
static CURRENT: AtomicUsize = AtomicUsize::new(0);

fn parse(raw: &str) -> Table {
    let mut table = Table::new();
    let mut key = String::new();
    for line in raw.lines() {
        let text = line.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        if line.starts_with(char::is_whitespace) {
            if let Some((code, value)) = text.split_once(':') {
                table
                    .entry(key.clone())
                    .or_default()
                    .insert(code.trim().to_owned(), value.trim().trim_matches('"').to_owned());
            }
        } else {
            key = text.trim_end_matches(':').to_owned();
        }
    }
    table
}

/// Change the current lang.
pub fn set(lang: Lang) {
    CURRENT.store(lang.index(), Ordering::Relaxed);
}

pub fn t(key: &str) -> String {
    let table = TABLE.get_or_init(|| parse(include_str!("../locales.yml")));
    let code = Lang::from_index(CURRENT.load(Ordering::Relaxed)).code();
    table
        .get(key)
        .and_then(|langs| langs.get(code).or_else(|| langs.get(FALLBACK)))
        .cloned()
        .unwrap_or_else(|| key.to_owned())
}

#[macro_export]
macro_rules! tr {
    ($key:expr) => { $crate::i18n::t($key) };
    ($key:expr, $($name:ident = $val:expr),+ $(,)?) => {{
        let mut text = $crate::i18n::t($key);
        $( text = text.replace(concat!("{", stringify!($name), "}"), &($val).to_string()); )+
        text
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translations_switch_and_interpolate() {
        set(Lang::Spanish);
        assert_eq!(t("nav.library"), "Biblioteca");

        set(Lang::English);
        assert_eq!(t("nav.library"), "Library");
        assert_eq!(
            crate::tr!("library.subtitle.one", count = 1),
            "1 video · click a thumbnail to apply it"
        );
        assert_eq!(
            crate::tr!("library.no_results.message", query = "rain"),
            "No videos match the search rain"
        );

        set(Lang::Spanish);
    }

    #[test]
    fn every_locale_key_has_both_languages() {
        let table = TABLE.get_or_init(|| parse(include_str!("../locales.yml")));
        for (key, translations) in table {
            assert!(
                translations.contains_key("es"),
                "missing Spanish translation: {key}"
            );
            assert!(
                translations.contains_key("en"),
                "missing English translation: {key}"
            );
        }
    }
}