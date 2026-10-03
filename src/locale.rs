#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Locale {
    #[default]
    ZhCn,
    En,
}

pub(crate) fn parse_locale(value: Option<&str>) -> Locale {
    match value {
        Some("en") => Locale::En,
        _ => Locale::ZhCn,
    }
}

impl Locale {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::ZhCn => "zh-CN",
            Self::En => "en",
        }
    }

    pub(crate) const fn select(self, zh: &'static str, en: &'static str) -> &'static str {
        match self {
            Self::ZhCn => zh,
            Self::En => en,
        }
    }

    pub(crate) const fn home(self) -> &'static str {
        self.select("首页", "Home")
    }

    pub(crate) const fn writing(self) -> &'static str {
        self.select("文章", "Writing")
    }

    pub(crate) const fn focus(self) -> &'static str {
        self.select("关注领域", "Focus")
    }

    pub(crate) const fn about(self) -> &'static str {
        self.select("关于", "About")
    }

    pub(crate) const fn not_found(self) -> &'static str {
        self.select("页面未找到", "Page not found")
    }
}

/// Browser document cannot be accessed; the user's selected language remains active for this page.
#[cfg(feature = "hydrate")]
#[derive(Debug)]
pub(crate) enum LocalePersistError {
    /// The browser did not provide an HTML document to write a cookie.
    DocumentUnavailable,
    /// The browser rejected the cookie write; the JS exception remains available to the caller.
    CookieWriteFailed(wasm_bindgen::JsValue),
}

#[cfg(feature = "hydrate")]
pub(crate) fn persist_locale(locale: Locale) -> Result<(), LocalePersistError> {
    use wasm_bindgen::JsCast;

    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or(LocalePersistError::DocumentUnavailable)?;
    let is_secure = document.location().is_some_and(|location| {
        location
            .protocol()
            .is_ok_and(|protocol| protocol == "https:")
    });
    let secure = if is_secure { "; Secure" } else { "" };
    let cookie = format!(
        "mcb-lang={}; Path=/; SameSite=Lax; Max-Age=31536000{secure}",
        locale.as_str()
    );
    let html_document = document
        .dyn_into::<web_sys::HtmlDocument>()
        .map_err(|_| LocalePersistError::DocumentUnavailable)?;
    html_document
        .set_cookie(&cookie)
        .map_err(LocalePersistError::CookieWriteFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_locale_supported_values_returns_locale() {
        assert_eq!(parse_locale(Some("zh-CN")), Locale::ZhCn);
        assert_eq!(parse_locale(Some("en")), Locale::En);
        assert_eq!(Locale::ZhCn.as_str(), "zh-CN");
        assert_eq!(Locale::En.as_str(), "en");
    }

    #[test]
    fn parse_locale_invalid_values_returns_chinese() {
        for value in [
            None,
            Some(""),
            Some("EN"),
            Some("zh-cn"),
            Some("en-US"),
            Some("en;debug"),
        ] {
            assert_eq!(parse_locale(value), Locale::ZhCn);
        }
    }
}
