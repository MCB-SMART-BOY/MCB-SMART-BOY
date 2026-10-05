use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use crate::reading::{ReadingRoute, resolve_reading_route};
use crate::{content::Post, locale::Locale, markdown::HeadingEntry};

#[cfg(feature = "hydrate")]
use std::{cell::RefCell, rc::Rc};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure};

#[derive(Clone, Copy)]
struct ArticleOutlineContext(RwSignal<Option<&'static str>>);

#[cfg(feature = "hydrate")]
struct HeadingNode {
    id: &'static str,
    element: web_sys::Element,
}

#[cfg(feature = "hydrate")]
struct ScrollTracker {
    headings: Vec<HeadingNode>,
    topbar: Option<web_sys::Element>,
    scroll_root: web_sys::Element,
    window: web_sys::Window,
    listener: Closure<dyn FnMut(web_sys::Event)>,
    pending: Option<AnimationFrameRequestHandle>,
}

#[cfg(feature = "hydrate")]
type TrackerSlot = Rc<RefCell<Option<ScrollTracker>>>;

#[cfg(feature = "hydrate")]
const TRACKED_EVENTS: [&str; 4] = ["scroll", "resize", "hashchange", "popstate"];
#[cfg(feature = "hydrate")]
const HEADING_CLEARANCE_PX: f64 = 16.0;
#[cfg(feature = "hydrate")]
const SCROLL_POSITION_TOLERANCE_PX: f64 = 1.0;

#[cfg(feature = "hydrate")]
fn clear_tracker(slot: &TrackerSlot) {
    if let Some(mut tracker) = slot.borrow_mut().take() {
        if let Some(handle) = tracker.pending.take() {
            handle.cancel();
        }
        for name in TRACKED_EVENTS {
            if let Err(source) = tracker.window.remove_event_listener_with_callback(
                name,
                tracker.listener.as_ref().unchecked_ref(),
            ) {
                web_sys::console::error_2(
                    &format!("Failed to detach article {name} listener").into(),
                    &source,
                );
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn read_active_heading(tracker: &ScrollTracker) -> Option<&'static str> {
    let scroll_top = tracker.scroll_root.scroll_top();
    let max_scroll = tracker.scroll_root.scroll_height() - tracker.scroll_root.client_height();
    if scroll_top > 0
        && f64::from(scroll_top) >= f64::from(max_scroll) - SCROLL_POSITION_TOLERANCE_PX
    {
        return tracker.headings.last().map(|heading| heading.id);
    }
    let threshold = tracker
        .topbar
        .as_ref()
        .map(|topbar| topbar.get_bounding_client_rect().bottom())
        .unwrap_or_default()
        + HEADING_CLEARANCE_PX
        + SCROLL_POSITION_TOLERANCE_PX;
    let mut active = tracker.headings.first()?.id;
    for heading in &tracker.headings {
        if heading.element.get_bounding_client_rect().top() > threshold {
            break;
        }
        active = heading.id;
    }
    Some(active)
}

#[cfg(feature = "hydrate")]
fn schedule_heading_sample(slot: &TrackerSlot, context: ArticleOutlineContext) {
    if slot
        .borrow()
        .as_ref()
        .is_none_or(|tracker| tracker.pending.is_some())
    {
        return;
    }
    let pending_slot = Rc::clone(slot);
    match request_animation_frame_with_handle(move || {
        let selected = pending_slot.borrow_mut().as_mut().and_then(|tracker| {
            tracker.pending.take();
            read_active_heading(tracker)
        });
        if context.0.get_untracked() != selected {
            context.0.set(selected);
        }
    }) {
        Ok(handle) => {
            if let Some(tracker) = slot.borrow_mut().as_mut() {
                tracker.pending = Some(handle);
            } else {
                handle.cancel();
            }
        }
        Err(source) => {
            web_sys::console::error_2(&"Failed to sample article headings".into(), &source)
        }
    }
}

#[cfg(feature = "hydrate")]
fn find_heading_nodes(
    post: &'static Post,
    document: &web_sys::Document,
) -> Option<Vec<HeadingNode>> {
    let article = match document.query_selector(".article-body") {
        Ok(Some(article)) => article,
        Ok(None) => return None,
        Err(source) => {
            web_sys::console::error_2(&"Failed to locate article body for outline".into(), &source);
            return None;
        }
    };
    let mut nodes = Vec::with_capacity(post.render().headings.len());
    for heading in &post.render().headings {
        let Some(element) = document.get_element_by_id(&heading.id) else {
            return None;
        };
        if !article.contains(Some(&element)) {
            return None;
        }
        nodes.push(HeadingNode {
            id: &heading.id,
            element,
        });
    }
    Some(nodes)
}

#[cfg(feature = "hydrate")]
fn install_scroll_tracker(post: &'static Post, slot: &TrackerSlot, context: ArticleOutlineContext) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(scroll_root) = document.document_element() else {
        return;
    };
    let Some(headings) = find_heading_nodes(post, &document) else {
        return;
    };
    let topbar = match document.query_selector(".site-topbar") {
        Ok(topbar) => topbar,
        Err(source) => {
            web_sys::console::error_2(
                &"Failed to locate topbar for article outline".into(),
                &source,
            );
            None
        }
    };
    let listener_slot = Rc::clone(slot);
    let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
        schedule_heading_sample(&listener_slot, context);
    });
    for name in TRACKED_EVENTS {
        if let Err(source) =
            window.add_event_listener_with_callback(name, listener.as_ref().unchecked_ref())
        {
            web_sys::console::error_2(
                &format!("Failed to attach article {name} listener").into(),
                &source,
            );
        }
    }
    *slot.borrow_mut() = Some(ScrollTracker {
        headings,
        topbar,
        scroll_root,
        window,
        listener,
        pending: None,
    });
    schedule_heading_sample(slot, context);
}

pub(super) fn provide_article_outline_context(pathname: Memo<String>) {
    let context = ArticleOutlineContext(RwSignal::new(None));
    provide_context(context);
    #[cfg(feature = "hydrate")]
    {
        let slot: TrackerSlot = Rc::new(RefCell::new(None));
        let pending = StoredValue::new_local(RefCell::new(None::<AnimationFrameRequestHandle>));
        let route_slot = Rc::clone(&slot);
        Effect::new(move |_| {
            let path = pathname.get();
            clear_tracker(&route_slot);
            if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
                handle.cancel();
            }
            context.0.set(None);
            let Some(ReadingRoute::Article(post)) = resolve_reading_route(&path) else {
                return;
            };
            if post.render().headings.is_empty() {
                return;
            }
            let next_slot = Rc::clone(&route_slot);
            match request_animation_frame_with_handle(move || {
                pending.with_value(|pending| pending.borrow_mut().take());
                install_scroll_tracker(post, &next_slot, context);
            }) {
                Ok(handle) => pending.with_value(|pending| *pending.borrow_mut() = Some(handle)),
                Err(source) => {
                    web_sys::console::error_2(&"Failed to bind article outline".into(), &source)
                }
            }
        });
        let cleanup_slot = StoredValue::new_local(slot);
        on_cleanup(move || {
            if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
                handle.cancel();
            }
            cleanup_slot.with_value(clear_tracker);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = pathname;
}

#[component]
fn OutlineLinks(
    headings: &'static [HeadingEntry],
    min_level: u8,
    context: ArticleOutlineContext,
    post: Option<&'static Post>,
) -> impl IntoView {
    headings
        .iter()
        .map(|heading| {
            let href = match post {
                Some(post) => format!("{}#{}", post.path, heading.id),
                None => format!("#{}", heading.id),
            };
            let depth = heading.level.saturating_sub(min_level);
            let text = heading.text.as_str();
            view! {
                <a
                    class="article-outline-link"
                    href=href
                    style=format!("--outline-depth: {depth}")
                    aria-current=move || (context.0.get() == Some(heading.id.as_str()))
                        .then_some("location")
                    lang="zh-CN"
                >{text}</a>
            }
        })
        .collect_view()
}

#[component]
fn OutlineContent(
    headings: &'static [HeadingEntry],
    min_level: u8,
    context: ArticleOutlineContext,
    locale: RwSignal<Locale>,
    post: Option<&'static Post>,
) -> impl IntoView {
    if headings.is_empty() {
        view! { <p class="article-outline-empty">{move || locale.get().select("本文暂无小标题", "No section headings")}</p> }.into_any()
    } else {
        view! {
            <div class="article-outline-links">
                <OutlineLinks headings min_level context post/>
            </div>
        }
        .into_any()
    }
}

#[component]
pub(crate) fn ArticleOutline(headings: &'static [HeadingEntry]) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let context = use_context::<ArticleOutlineContext>()
        .unwrap_or_else(|| ArticleOutlineContext(RwSignal::new(None)));
    let min_level = headings
        .iter()
        .map(|heading| heading.level)
        .min()
        .unwrap_or(1);

    view! {
        <details class="article-outline-compact">
            <summary>{move || locale.get().select("本文目录", "On this page")}</summary>
            <OutlineContent headings min_level context locale post=None/>
        </details>
    }
}

#[component]
pub(crate) fn ArticleSectionLinks(post: &'static Post) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let headings = post.render().headings.as_slice();
    let min_level = headings
        .iter()
        .map(|heading| heading.level)
        .min()
        .unwrap_or(1);
    let context = use_context::<ArticleOutlineContext>()
        .unwrap_or_else(|| ArticleOutlineContext(RwSignal::new(None)));
    view! {
        <nav class="reading-navigation-paragraphs" aria-label=move || locale.get().select("段落目录", "Article sections")>
            <OutlineContent headings min_level context locale post=Some(post)/>
        </nav>
    }
}
