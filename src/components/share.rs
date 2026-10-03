use leptos::{html, prelude::*};

use super::icons::{Icon, IconKind};
use crate::locale::Locale;

#[cfg(feature = "hydrate")]
use leptos_router::hooks::use_location;
#[cfg(feature = "hydrate")]
use leptos_router::location::Location;
#[cfg(feature = "hydrate")]
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, JsValue};

// SSR renders an empty status; these outcomes are only produced by hydrated browser actions.
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
#[derive(Clone, Copy)]
enum ShareFeedback {
    Invoked,
    Copied,
    LinkUnavailable,
}

// SSR renders no manual panel; its reason is only produced by hydrated browser actions.
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
#[derive(Clone, Copy)]
enum ManualReason {
    ShareFailed,
    ClipboardFailed,
}

#[derive(Clone)]
struct ManualPanel {
    url: String,
    reason: ManualReason,
}

#[derive(Clone, Copy)]
struct ShareState {
    is_sharing: RwSignal<bool>,
    feedback: RwSignal<Option<ShareFeedback>>,
    panel: RwSignal<Option<ManualPanel>>,
    #[cfg(feature = "hydrate")]
    current_url: RwSignal<String>,
    #[cfg(feature = "hydrate")]
    current_route: RwSignal<Option<(String, String, String)>>,
}

impl ShareState {
    #[cfg(feature = "hydrate")]
    fn reset_for_url(self, url: String, generation: &AtomicU64) {
        if self.current_url.get_untracked() == url {
            return;
        }
        self.current_url.set(url);
        self.invalidate(generation);
    }

    #[cfg(feature = "hydrate")]
    fn reset_for_route(self, route: (String, String, String), generation: &AtomicU64) {
        if self.current_route.get_untracked().as_ref() == Some(&route) {
            return;
        }
        self.current_route.set(Some(route));
        self.invalidate(generation);
    }

    #[cfg(feature = "hydrate")]
    fn invalidate(self, generation: &AtomicU64) {
        generation.fetch_add(1, Ordering::Relaxed);
        self.is_sharing.set(false);
        self.feedback.set(None);
        self.panel.set(None);
    }
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy)]
enum ShareOperation {
    WebShare,
    Clipboard,
}

#[cfg(feature = "hydrate")]
enum ShareError {
    /// The browser window or document cannot be reached.
    BrowserUnavailable,
    /// Reading the current URL failed with a JS exception.
    LinkUnavailable(JsValue),
    /// The requested browser capability does not exist.
    Unavailable(ShareOperation),
    /// A browser method threw or rejected its promise; keep the original JS reason.
    Invocation {
        operation: ShareOperation,
        source: JsValue,
    },
    /// A browser method returned something other than a promise.
    InvalidPromise(ShareOperation),
    /// A browser capability is present but is not callable.
    InvalidMethod(ShareOperation),
}

#[cfg(feature = "hydrate")]
fn js_error_kind(source: &JsValue) -> &'static str {
    let name = source
        .dyn_ref::<web_sys::DomException>()
        .map(|error| error.name())
        .or_else(|| {
            js_sys::Reflect::get(source, &JsValue::from_str("name"))
                .ok()
                .and_then(|name| name.as_string())
        });
    match name.as_deref() {
        Some("AbortError") => "AbortError",
        Some("NotAllowedError") => "NotAllowedError",
        Some("SecurityError") => "SecurityError",
        Some("TypeError") => "TypeError",
        Some("DataError") => "DataError",
        Some("InvalidStateError") => "InvalidStateError",
        _ => "other",
    }
}

#[cfg(feature = "hydrate")]
impl ShareError {
    fn is_cancelled(&self) -> bool {
        matches!(self, Self::Invocation { operation: ShareOperation::WebShare, source }
            if js_error_kind(source) == "AbortError")
    }

    fn category(&self) -> &'static str {
        match self {
            Self::BrowserUnavailable => "browser unavailable",
            Self::LinkUnavailable(_) => "location unavailable",
            Self::Unavailable(ShareOperation::WebShare) => "web share unavailable",
            Self::Unavailable(ShareOperation::Clipboard) => "clipboard unavailable",
            Self::Invocation {
                operation: ShareOperation::WebShare,
                ..
            } => "web share rejected",
            Self::Invocation {
                operation: ShareOperation::Clipboard,
                ..
            } => "clipboard rejected",
            Self::InvalidPromise(ShareOperation::WebShare) => "web share returned no promise",
            Self::InvalidPromise(ShareOperation::Clipboard) => "clipboard returned no promise",
            Self::InvalidMethod(ShareOperation::WebShare) => "web share method invalid",
            Self::InvalidMethod(ShareOperation::Clipboard) => "clipboard method invalid",
        }
    }

    fn reason_kind(&self) -> &'static str {
        match self {
            Self::LinkUnavailable(source) | Self::Invocation { source, .. } => {
                js_error_kind(source)
            }
            _ => "none",
        }
    }

    fn manual_reason(&self) -> ManualReason {
        match self {
            Self::Unavailable(ShareOperation::Clipboard)
            | Self::Invocation {
                operation: ShareOperation::Clipboard,
                ..
            }
            | Self::InvalidPromise(ShareOperation::Clipboard)
            | Self::InvalidMethod(ShareOperation::Clipboard) => ManualReason::ClipboardFailed,
            _ => ManualReason::ShareFailed,
        }
    }
}

#[cfg(feature = "hydrate")]
struct PendingShare {
    promise: js_sys::Promise,
    operation: ShareOperation,
}

#[cfg(feature = "hydrate")]
fn find_method(
    object: &JsValue,
    name: &str,
    operation: ShareOperation,
) -> Result<Option<js_sys::Function>, ShareError> {
    let value = js_sys::Reflect::get(object, &JsValue::from_str(name))
        .map_err(|source| ShareError::Invocation { operation, source })?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    value
        .dyn_into::<js_sys::Function>()
        .map(Some)
        .map_err(|_| ShareError::InvalidMethod(operation))
}

#[cfg(feature = "hydrate")]
fn invoke_method(
    method: &js_sys::Function,
    receiver: &JsValue,
    data: &JsValue,
    operation: ShareOperation,
) -> Result<PendingShare, ShareError> {
    let result = method
        .call1(receiver, data)
        .map_err(|source| ShareError::Invocation { operation, source })?;
    let promise = result
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| ShareError::InvalidPromise(operation))?;
    Ok(PendingShare { promise, operation })
}

#[cfg(feature = "hydrate")]
fn invoke_share(url: &str, title: &str) -> Result<PendingShare, ShareError> {
    let navigator = web_sys::window()
        .ok_or(ShareError::BrowserUnavailable)?
        .navigator();
    if let Some(method) = find_method(navigator.as_ref(), "share", ShareOperation::WebShare)? {
        let data = web_sys::ShareData::new();
        data.set_title(title);
        data.set_url(url);
        return invoke_method(
            &method,
            navigator.as_ref(),
            data.as_ref(),
            ShareOperation::WebShare,
        );
    }
    let clipboard = js_sys::Reflect::get(navigator.as_ref(), &JsValue::from_str("clipboard"))
        .map_err(|source| ShareError::Invocation {
            operation: ShareOperation::Clipboard,
            source,
        })?;
    if clipboard.is_null() || clipboard.is_undefined() {
        return Err(ShareError::Unavailable(ShareOperation::Clipboard));
    }
    let method = find_method(&clipboard, "writeText", ShareOperation::Clipboard)?
        .ok_or(ShareError::Unavailable(ShareOperation::Clipboard))?;
    invoke_method(
        &method,
        &clipboard,
        &JsValue::from_str(url),
        ShareOperation::Clipboard,
    )
}

#[cfg(feature = "hydrate")]
fn capture_share_target() -> Result<(String, String), ShareError> {
    let window = web_sys::window().ok_or(ShareError::BrowserUnavailable)?;
    let url = window
        .location()
        .href()
        .map_err(ShareError::LinkUnavailable)?;
    let document = window.document().ok_or(ShareError::BrowserUnavailable)?;
    Ok((url, document.title()))
}

#[cfg(feature = "hydrate")]
fn route_key(location: &Location) -> (String, String, String) {
    (
        location.pathname.get_untracked(),
        location.search.get_untracked(),
        location.hash.get_untracked(),
    )
}

#[cfg(feature = "hydrate")]
fn is_current_request(
    url: &str,
    route: &(String, String, String),
    location: &Location,
    generation: u64,
    counter: &AtomicU64,
    is_alive: &AtomicBool,
) -> bool {
    is_alive.load(Ordering::Relaxed)
        && counter.load(Ordering::Relaxed) == generation
        && route_key(location) == *route
        && web_sys::window()
            .and_then(|window| window.location().href().ok())
            .as_deref()
            == Some(url)
}

#[cfg(feature = "hydrate")]
fn finish_share(state: ShareState, url: String, outcome: Result<ShareOperation, ShareError>) {
    state.is_sharing.set(false);
    match outcome {
        Ok(ShareOperation::WebShare) => state.feedback.set(Some(ShareFeedback::Invoked)),
        Ok(ShareOperation::Clipboard) => state.feedback.set(Some(ShareFeedback::Copied)),
        Err(error) if error.is_cancelled() => {}
        Err(error) => {
            web_sys::console::error_1(
                &format!(
                    "Share operation failed: {} ({})",
                    error.category(),
                    error.reason_kind()
                )
                .into(),
            );
            if matches!(
                &error,
                ShareError::LinkUnavailable(_) | ShareError::BrowserUnavailable
            ) {
                state.feedback.set(Some(ShareFeedback::LinkUnavailable));
            } else {
                state.panel.set(Some(ManualPanel {
                    url,
                    reason: error.manual_reason(),
                }));
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn start_share(
    state: ShareState,
    counter: Arc<AtomicU64>,
    is_alive: Arc<AtomicBool>,
    location: Location,
) {
    if state.is_sharing.get_untracked() {
        return;
    }
    let (url, title) = match capture_share_target() {
        Ok(target) => target,
        Err(error) => {
            finish_share(state, String::new(), Err(error));
            return;
        }
    };
    state.reset_for_url(url.clone(), &counter);
    let route = route_key(&location);
    let generation = counter.fetch_add(1, Ordering::Relaxed) + 1;
    state.feedback.set(None);
    state.panel.set(None);
    state.is_sharing.set(true);
    // The API call must happen in this click handler, before any await loses user activation.
    let attempt = invoke_share(&url, &title);
    match attempt {
        Ok(PendingShare { promise, operation }) => wasm_bindgen_futures::spawn_local(async move {
            let result = wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map(|_| operation)
                .map_err(|source| ShareError::Invocation { operation, source });
            if is_current_request(&url, &route, &location, generation, &counter, &is_alive) {
                finish_share(state, url, result);
            }
        }),
        Err(error) => {
            if is_current_request(&url, &route, &location, generation, &counter, &is_alive) {
                finish_share(state, url, Err(error));
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn observe_navigation(state: ShareState, counter: &AtomicU64) {
    if let Some(url) = web_sys::window().and_then(|window| window.location().href().ok()) {
        state.reset_for_url(url, counter);
    }
}

#[cfg(feature = "hydrate")]
fn watch_navigation(state: ShareState, location: Location) -> (Arc<AtomicU64>, Arc<AtomicBool>) {
    let counter = Arc::new(AtomicU64::new(0));
    let is_alive = Arc::new(AtomicBool::new(true));
    state.reset_for_route(route_key(&location), &counter);
    observe_navigation(state, &counter);
    let reactive_counter = counter.clone();
    Effect::new(move || {
        let route = (
            location.pathname.get(),
            location.search.get(),
            location.hash.get(),
        );
        state.reset_for_route(route, &reactive_counter);
        observe_navigation(state, &reactive_counter);
    });
    let hash_counter = counter.clone();
    let hash_listener = window_event_listener(leptos::ev::hashchange, move |_| {
        observe_navigation(state, &hash_counter);
    });
    let pop_counter = counter.clone();
    let pop_listener = window_event_listener(leptos::ev::popstate, move |_| {
        observe_navigation(state, &pop_counter);
    });
    on_cleanup({
        let counter = counter.clone();
        let is_alive = is_alive.clone();
        move || {
            is_alive.store(false, Ordering::Relaxed);
            counter.fetch_add(1, Ordering::Relaxed);
            hash_listener.remove();
            pop_listener.remove();
        }
    });
    (counter, is_alive)
}

#[component]
fn SharePanel(
    state: ShareState,
    locale: RwSignal<Locale>,
    button: NodeRef<html::Button>,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = &button;
    let input = NodeRef::<html::Input>::new();
    #[cfg(feature = "hydrate")]
    Effect::new(move || {
        if let Some(field) = input.get() {
            let _ = field.focus();
            field.select();
        }
    });
    let close = move || {
        state.panel.set(None);
        #[cfg(feature = "hydrate")]
        if let Some(button) = button.get_untracked() {
            let _ = button.focus();
        }
    };
    view! {
        <div id="share-manual-panel" class="share-panel" on:keydown=move |event| {
            if event.key() == "Escape" {
                event.prevent_default();
                event.stop_propagation();
                close();
            }
        }>
            <p role="alert">{move || match state.panel.get().map(|panel| panel.reason) {
                Some(ManualReason::ShareFailed) => locale.get().select("分享失败，请手动复制链接", "Sharing failed. Copy this link manually."),
                Some(ManualReason::ClipboardFailed) => locale.get().select("无法复制链接，请手动复制链接", "Unable to copy. Copy this link manually."),
                None => "",
            }}</p>
            <label for="share-manual-url">{move || locale.get().select("当前页面链接", "Current page link")}</label>
            <input id="share-manual-url" node_ref=input type="text" readonly value=move || state.panel.get().map(|panel| panel.url).unwrap_or_default()/>
            <button class="share-panel-close" type="button" on:click=move |_| close()>
                {move || locale.get().select("关闭", "Close")}
            </button>
        </div>
    }
}

#[component]
pub(super) fn ShareButton(is_hydrated: RwSignal<bool>) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let state = ShareState {
        is_sharing: RwSignal::new(false),
        feedback: RwSignal::new(None),
        panel: RwSignal::new(None),
        #[cfg(feature = "hydrate")]
        current_url: RwSignal::new(String::new()),
        #[cfg(feature = "hydrate")]
        current_route: RwSignal::new(None),
    };
    let button = NodeRef::<html::Button>::new();
    #[cfg(feature = "hydrate")]
    let location = use_location();
    #[cfg(feature = "hydrate")]
    let (counter, is_alive) = watch_navigation(state, location.clone());
    view! {
        <div class="share-action">
            <button
                node_ref=button
                class="topbar-button share-button"
                type="button"
                disabled=move || !is_hydrated.get() || state.is_sharing.get()
                aria-label=move || locale.get().select("分享当前页面", "Share this page")
                title=move || locale.get().select("分享当前页面", "Share this page")
                aria-expanded=move || if state.panel.get().is_some() { "true" } else { "false" }
                aria-controls="share-manual-panel"
                on:click=move |_| {
                    #[cfg(feature = "hydrate")]
                    start_share(state, counter.clone(), is_alive.clone(), location.clone());
                }
            ><Icon kind=IconKind::Share/></button>
            <span class="sr-only" aria-live="polite">{move || match state.feedback.get() {
                Some(ShareFeedback::Invoked) => locale.get().select("已调用系统分享", "System sharing invoked"),
                Some(ShareFeedback::Copied) => locale.get().select("链接已复制", "Link copied"),
                Some(ShareFeedback::LinkUnavailable) => locale.get().select("无法获取当前页面链接", "Unable to get the current page link"),
                None => "",
            }}</span>
            <Show when=move || state.panel.get().is_some()>
                <SharePanel state locale button/>
            </Show>
        </div>
    }
}
