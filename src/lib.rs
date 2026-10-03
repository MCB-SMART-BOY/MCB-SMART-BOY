pub mod app;
pub mod components;
mod content;
mod locale;
mod markdown;
pub mod pages;
pub mod preferences;
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
            .and_then(|element| element.get_attribute("data-ui"))
            .as_deref(),
    );
    let locale = locale::parse_locale(
        root.as_ref()
            .and_then(|element| element.get_attribute("lang"))
            .as_deref(),
    );
    leptos::mount::hydrate_body(move || {
        provide_context(preferences);
        provide_context(locale);
        view! { <app::App/> }
    });
}
