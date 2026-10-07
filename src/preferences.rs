pub const SIDEBAR_DEFAULT_PX: u16 = 248;
pub const SIDEBAR_MIN_PX: u16 = 208;
pub const SIDEBAR_MAX_PX: u16 = 360;
pub const SIDEBAR_COLLAPSED_PX: u16 = 64;
pub const SIDEBAR_CONTENT_MIN_PX: u16 = 448;
pub const SIDEBAR_MOBILE_BREAKPOINT_PX: u16 = 768;
const MAX_COOKIE_BYTES: usize = 64;

/// Applies only the validated saved palette before CSS; hydration always reads immutable SSR attrs.
/// Deliberately leaves data-ui, lang, and the component tree untouched if WASM never loads.
/// Invariant: this grammar and its numeric bounds match `parse_preferences`.
#[cfg(feature = "ssr")]
pub(crate) const THEME_BOOTSTRAP: &str = r##"(function () {
  try {
    var MAX_COOKIE_BYTES = 64;
    var U16_MAX = 65535;
    var entries = document.cookie.split(";");
    var value = null;
    for (var index = 0; index < entries.length; index += 1) {
      var entry = entries[index].trim();
      if (entry.startsWith("mcb-ui=")) {
        value = entry.slice("mcb-ui=".length);
        break;
      }
    }
    if (value === null || value.length > MAX_COOKIE_BYTES) return;
    var match = /^(light|dark):[01]:([0-9]+)$/.exec(value);
    if (!match || Number(match[2]) > U16_MAX) return;
    var theme = match[1];
    document.documentElement.setAttribute("data-theme", theme);
    var meta = document.querySelector('meta[name="theme-color"]');
    if (meta) meta.setAttribute("content", theme === "dark" ? "#09070f" : "#fcfbf7");
  } catch (error) {
    console.error("Failed to restore saved theme: browser cookie or document unavailable", error);
  }
})();"##;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

impl Theme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub const fn color(self) -> &'static str {
        match self {
            Self::Light => "#fcfbf7",
            Self::Dark => "#09070f",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiPreferences {
    pub theme: Theme,
    pub is_sidebar_collapsed: bool,
    pub sidebar_width_px: u16,
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::Light,
            is_sidebar_collapsed: false,
            sidebar_width_px: SIDEBAR_DEFAULT_PX,
        }
    }
}

impl UiPreferences {
    pub(crate) fn cookie_value(self) -> String {
        format!(
            "{}:{}:{}",
            self.theme.as_str(),
            u8::from(self.is_sidebar_collapsed),
            self.sidebar_width_px.clamp(SIDEBAR_MIN_PX, SIDEBAR_MAX_PX)
        )
    }
}

/// Invalid, oversized, or malformed non-critical UI cookies are ignored as a whole.
pub(crate) fn parse_preferences(value: Option<&str>) -> UiPreferences {
    let Some(value) = value.filter(|value| value.len() <= MAX_COOKIE_BYTES) else {
        return UiPreferences::default();
    };
    let mut parts = value.split(':');
    let (Some(theme), Some(collapsed), Some(width), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return UiPreferences::default();
    };
    let theme = match theme {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => return UiPreferences::default(),
    };
    let is_sidebar_collapsed = match collapsed {
        "0" => false,
        "1" => true,
        _ => return UiPreferences::default(),
    };
    if width.is_empty() || !width.bytes().all(|digit| digit.is_ascii_digit()) {
        return UiPreferences::default();
    }
    let Ok(sidebar_width_px) = width.parse::<u16>() else {
        return UiPreferences::default();
    };
    UiPreferences {
        theme,
        is_sidebar_collapsed,
        sidebar_width_px: sidebar_width_px.clamp(SIDEBAR_MIN_PX, SIDEBAR_MAX_PX),
    }
}
/// Finds an exact cookie name without accepting similarly prefixed keys.
pub(crate) fn find_cookie_value<'a>(cookie_header: &'a str, name: &str) -> Option<&'a str> {
    cookie_header.split(';').find_map(|entry| {
        let (key, value) = entry.trim().split_once('=')?;
        (key == name).then_some(value)
    })
}

#[cfg(feature = "hydrate")]
pub(crate) fn read_browser_cookies() -> Option<String> {
    use wasm_bindgen::JsCast;

    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        web_sys::console::error_1(
            &"Failed to restore browser preferences: document unavailable".into(),
        );
        return None;
    };
    let Ok(document) = document.dyn_into::<web_sys::HtmlDocument>() else {
        web_sys::console::error_1(
            &"Failed to restore browser preferences: HTML document unavailable".into(),
        );
        return None;
    };
    match document.cookie() {
        Ok(cookie) => Some(cookie),
        Err(source) => {
            web_sys::console::error_2(
                &"Failed to restore browser preferences: cookie read rejected".into(),
                &source,
            );
            None
        }
    }
}

#[cfg(feature = "hydrate")]
pub(crate) fn persist_preferences(preferences: UiPreferences) {
    use wasm_bindgen::JsCast;
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        web_sys::console::error_1(&"无法保存界面偏好：浏览器文档不可用".into());
        return;
    };
    let is_secure = document.location().is_some_and(|location| {
        location
            .protocol()
            .is_ok_and(|protocol| protocol == "https:")
    });
    let secure = if is_secure { "; Secure" } else { "" };
    let cookie = format!(
        "mcb-ui={}; Path=/; SameSite=Lax; Max-Age=31536000{secure}",
        preferences.cookie_value()
    );
    let Ok(document) = document.dyn_into::<web_sys::HtmlDocument>() else {
        web_sys::console::error_1(&"无法保存界面偏好：HTML 文档不可用".into());
        return;
    };
    if let Err(source) = document.set_cookie(&cookie) {
        web_sys::console::error_2(&"无法保存界面偏好：cookie 写入失败".into(), &source);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_preferences_valid_cookie_clamps_width() {
        assert_eq!(
            parse_preferences(Some("dark:1:409")),
            UiPreferences {
                theme: Theme::Dark,
                is_sidebar_collapsed: true,
                sidebar_width_px: SIDEBAR_MAX_PX,
            }
        );
        assert_eq!(
            parse_preferences(Some("light:0:1")).sidebar_width_px,
            SIDEBAR_MIN_PX
        );
    }

    #[test]
    fn parse_preferences_invalid_cookie_uses_defaults() {
        for cookie in [
            "dark:2:248",
            "dark:1:-1",
            "dark:1:65536",
            "dark:1:248:extra",
            "dark:1:",
            "dark:1:２４８",
            "dark:1:248xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
        ] {
            assert_eq!(parse_preferences(Some(cookie)), UiPreferences::default());
        }
        assert_eq!(parse_preferences(None), UiPreferences::default());
    }

    #[test]
    fn find_cookie_value_exact_name_ignores_unrelated_entries() {
        let cookies = "mcb-ui-extra=dark:1:360; mcb-lang=en; mcb-ui=dark:1:360";
        assert_eq!(find_cookie_value(cookies, "mcb-ui"), Some("dark:1:360"));
        assert_eq!(find_cookie_value(cookies, "mcb-lang"), Some("en"));
        assert_eq!(find_cookie_value(cookies, "missing"), None);
    }

    #[test]
    fn cookie_value_uses_canonical_bounded_format() {
        assert_eq!(UiPreferences::default().cookie_value(), "light:0:248");
        assert_eq!(
            UiPreferences {
                theme: Theme::Dark,
                is_sidebar_collapsed: true,
                sidebar_width_px: 999
            }
            .cookie_value(),
            "dark:1:360"
        );
    }
}
