pub const SIDEBAR_DEFAULT_PX: u16 = 248;
pub const SIDEBAR_MIN_PX: u16 = 208;
pub const SIDEBAR_MAX_PX: u16 = 360;
pub const SIDEBAR_COLLAPSED_PX: u16 = 64;
pub const SIDEBAR_CONTENT_MIN_PX: u16 = 448;
pub const SIDEBAR_MOBILE_BREAKPOINT_PX: u16 = 768;
const MAX_COOKIE_BYTES: usize = 64;

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
            Self::Light => "#f7faff",
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
    if document.set_cookie(&cookie).is_err() {
        web_sys::console::error_1(&"无法保存界面偏好：cookie 写入失败".into());
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
