use i18n_embed::LanguageLoader;
use i18n_embed::fluent::FluentLanguageLoader;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use unic_langid::{LanguageIdentifier, langid};

#[derive(RustEmbed)]
#[folder = "locales"]
struct LocaleAssets;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(super) enum UiLanguage {
    #[serde(rename = "en-US", alias = "english", alias = "en-us")]
    English,
    #[serde(rename = "zh-CN", alias = "simplified-chinese", alias = "zh-cn")]
    #[default]
    SimplifiedChinese,
}

impl UiLanguage {
    pub(super) const fn available() -> &'static [Self] {
        &[Self::SimplifiedChinese, Self::English]
    }

    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::SimplifiedChinese => "简体中文",
        }
    }

    fn language_identifier(self) -> LanguageIdentifier {
        match self {
            Self::English => langid!("en-US"),
            Self::SimplifiedChinese => langid!("zh-CN"),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Translator {
    loader: &'static FluentLanguageLoader,
}

impl Translator {
    pub(super) fn new(language: UiLanguage) -> Self {
        Self {
            loader: cached_loader(language),
        }
    }

    pub(super) fn text(&self, key: &str) -> String {
        if self.loader.has(key) {
            self.loader.get(key)
        } else {
            key.to_string()
        }
    }

    pub(super) fn text_args(&self, key: &str, args: &[(&str, String)]) -> String {
        if !self.loader.has(key) {
            return key.to_string();
        }
        let args = args
            .iter()
            .map(|(key, value)| (*key, value.clone()))
            .collect();
        self.loader.get_args(key, args)
    }
}

fn cached_loader(language: UiLanguage) -> &'static FluentLanguageLoader {
    static ENGLISH: OnceLock<FluentLanguageLoader> = OnceLock::new();
    static SIMPLIFIED_CHINESE: OnceLock<FluentLanguageLoader> = OnceLock::new();

    match language {
        UiLanguage::English => ENGLISH.get_or_init(|| load_translator(language)),
        UiLanguage::SimplifiedChinese => {
            SIMPLIFIED_CHINESE.get_or_init(|| load_translator(language))
        }
    }
}

fn load_translator(language: UiLanguage) -> FluentLanguageLoader {
    let loader =
        FluentLanguageLoader::new("ui", UiLanguage::SimplifiedChinese.language_identifier());
    let mut languages = vec![language.language_identifier()];
    if language != UiLanguage::SimplifiedChinese {
        languages.push(UiLanguage::SimplifiedChinese.language_identifier());
    }
    let _ = loader.load_languages(&LocaleAssets, &languages);
    loader.set_use_isolating(false);
    loader
}

pub(super) fn args<const N: usize>(args: [(&str, String); N]) -> [(&str, String); N] {
    args
}

#[cfg(test)]
mod tests {
    use super::{Translator, UiLanguage, args};

    #[test]
    fn default_language_is_simplified_chinese() {
        assert_eq!(UiLanguage::default(), UiLanguage::SimplifiedChinese);
        assert_eq!(
            UiLanguage::available(),
            &[UiLanguage::SimplifiedChinese, UiLanguage::English]
        );
        assert_eq!(UiLanguage::SimplifiedChinese.label(), "简体中文");
        assert_eq!(UiLanguage::English.label(), "English");
    }

    #[test]
    fn translator_resolves_known_keys_in_both_languages() {
        let zh = Translator::new(UiLanguage::SimplifiedChinese);
        let en = Translator::new(UiLanguage::English);

        // Known keys must resolve to non-empty text in both languages,
        // and the two locales should produce distinct values.
        for key in ["main-menu-new-game", "settings-title"] {
            let zh_text = zh.text(key);
            let en_text = en.text(key);
            assert!(!zh_text.is_empty(), "zh text for '{key}' must not be empty");
            assert!(!en_text.is_empty(), "en text for '{key}' must not be empty");
            assert_ne!(zh_text, en_text, "zh/en must differ for key '{key}'");
        }
    }

    #[test]
    fn translator_formats_arguments_and_falls_back_to_key() {
        let en = Translator::new(UiLanguage::English);
        let zh = Translator::new(UiLanguage::SimplifiedChinese);

        // Argument substitution: the rendered text must embed the supplied value
        // and differ between languages (proving each locale's template is used).
        let count = "3".to_string();
        let en_rendered = en.text_args("message-turn-finished", &args([("count", count.clone())]));
        let zh_rendered = zh.text_args("message-turn-finished", &args([("count", count.clone())]));
        assert!(
            en_rendered.contains(&count),
            "English rendering must contain the substituted count"
        );
        assert!(
            zh_rendered.contains(&count),
            "Chinese rendering must contain the substituted count"
        );
        assert_ne!(en_rendered, zh_rendered, "en/zh rendered text must differ");

        // Fallback: an unknown key is returned verbatim.
        assert_eq!(en.text("missing-key-for-test"), "missing-key-for-test");
    }
}
