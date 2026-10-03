pub mod app;
pub mod components;
mod content;
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
    let bootstrap = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.document_element())
        .and_then(|root| root.get_attribute("data-ui"));
    let preferences = preferences::parse_preferences(bootstrap.as_deref());
    leptos::mount::hydrate_body(move || {
        provide_context(preferences);
        view! { <app::App/> }
    });
}
