// The combined landing sections need deeper compile-time view layout queries.
#![recursion_limit = "256"]

pub mod app;
pub mod components;
mod content;
#[cfg(feature = "ssr")]
pub mod export;
mod landing;
#[cfg(feature = "hydrate")]
mod landing_motion;
mod locale;
mod markdown;
pub mod pages;
pub mod preferences;
mod profile;
mod reading;
#[cfg(feature = "ssr")]
pub mod server;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use leptos::prelude::*;
    console_error_panic_hook::set_once();
    let root = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element());
    let preferences = preferences::parse_preferences(
        root.as_ref()
            .and_then(|element| element.get_attribute("data-ssr-ui"))
            .as_deref(),
    );
    let locale = locale::parse_locale(
        root.as_ref()
            .and_then(|element| element.get_attribute("data-ssr-lang"))
            .as_deref(),
    );
    leptos::mount::hydrate_body(move || {
        provide_context(preferences);
        provide_context(locale);
        view! { <app::App/> }
    });
}
