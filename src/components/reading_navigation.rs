use leptos::{html, prelude::*};
#[cfg(feature = "hydrate")]
use leptos_router::{NavigateOptions, hooks::use_navigate};
use leptos_router::{components::A, hooks::use_location};

use crate::{
    content::{BOOKS, Book, Chapter, Post, find_post, find_post_location},
    locale::Locale,
    reading::{ReadingRoute, build_chapter_path, resolve_reading_route},
};

use super::{
    article_outline::SidebarOutline,
    icons::{Icon, IconKind},
    sidebar::SidebarNavigation,
};

#[cfg(feature = "hydrate")]
use leptos::ev;
#[cfg(feature = "hydrate")]
use std::cell::{Cell, RefCell};
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, closure::Closure};

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum NavigationPage {
    Main,
    Books,
    Book,
    Chapter,
    Article,
}

impl NavigationPage {
    const COUNT: usize = 5;

    #[cfg(feature = "hydrate")]
    fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy)]
struct NavigationState {
    page: RwSignal<NavigationPage>,
    book: RwSignal<Option<&'static Book>>,
    chapter: RwSignal<Option<&'static Chapter>>,
    article: RwSignal<Option<&'static Post>>,
    focus_request: RwSignal<Option<NavigationPage>>,
}

impl NavigationState {
    fn change_page(self, page: NavigationPage, should_focus: bool) {
        self.page.set(page);
        if should_focus {
            self.focus_request.set(Some(page));
        }
    }

    fn select_route(self, route: Option<ReadingRoute>, should_focus: bool) -> NavigationPage {
        let page = match route {
            Some(ReadingRoute::Index) => NavigationPage::Books,
            Some(ReadingRoute::Book(book)) => {
                self.book.set(Some(book));
                NavigationPage::Book
            }
            Some(ReadingRoute::Chapter(book, chapter)) => {
                self.book.set(Some(book));
                self.chapter.set(Some(chapter));
                NavigationPage::Chapter
            }
            Some(ReadingRoute::Article(post)) => {
                self.article.set(Some(post));
                if let Some((book, chapter)) = find_post_location(post.slug) {
                    self.book.set(Some(book));
                    self.chapter.set(Some(chapter));
                } else {
                    self.book.set(None);
                    self.chapter.set(None);
                }
                NavigationPage::Article
            }
            None => NavigationPage::Main,
        };
        self.change_page(page, should_focus);
        page
    }
}

#[cfg(feature = "hydrate")]
fn navigate_root_page(
    state: NavigationState,
    page: NavigationPage,
    should_focus: bool,
    navigate: impl Fn(&str, NavigateOptions),
) {
    let href = match page {
        NavigationPage::Main => "/",
        NavigationPage::Books => "/writing",
        _ => return,
    };
    state.change_page(page, should_focus);
    navigate(href, NavigateOptions::default());
}

fn is_unmodified_click(event: &leptos::ev::MouseEvent) -> bool {
    event.button() == 0
        && !event.ctrl_key()
        && !event.meta_key()
        && !event.shift_key()
        && !event.alt_key()
}

#[cfg(feature = "hydrate")]
#[derive(Clone, Copy, Default)]
struct WheelGesture {
    last_at: f64,
    locked_until: f64,
    accumulated: f64,
}

#[cfg(feature = "hydrate")]
const WHEEL_LINE_MODE: u32 = 1;
#[cfg(feature = "hydrate")]
const WHEEL_PAGE_MODE: u32 = 2;
#[cfg(feature = "hydrate")]
const WHEEL_LINE_PX: f64 = 16.0;
#[cfg(feature = "hydrate")]
const WHEEL_THRESHOLD_PX: f64 = 36.0;
#[cfg(feature = "hydrate")]
const WHEEL_GESTURE_GAP_MS: f64 = 180.0;
#[cfg(feature = "hydrate")]
const WHEEL_FLIP_LOCK_MS: f64 = 560.0;

#[cfg(feature = "hydrate")]
fn is_scrollable_in_direction(top: i32, height: i32, viewport: i32, delta: f64) -> bool {
    let remaining = height - viewport;
    remaining > 1
        && if delta > 0.0 {
            top < remaining - 1
        } else {
            top > 1
        }
}

#[cfg(feature = "hydrate")]
fn has_focus_within(panel: NodeRef<html::Div>) -> bool {
    let active = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.active_element());
    active.is_some_and(|active| {
        panel
            .get_untracked()
            .is_some_and(|panel| panel.contains(Some(&active)))
    })
}

#[cfg(feature = "hydrate")]
fn handle_navigation_wheel(
    state: NavigationState,
    panel_ref: NodeRef<html::Div>,
    delta: f64,
    now: f64,
    gesture_cell: &Cell<WheelGesture>,
    navigate: impl Fn(&str, NavigateOptions),
) -> bool {
    let Some(panel) = panel_ref.get_untracked() else {
        return false;
    };
    let mut gesture = gesture_cell.get();
    if now < gesture.locked_until
        || (gesture.locked_until > 0.0 && now - gesture.last_at < WHEEL_GESTURE_GAP_MS)
    {
        gesture.last_at = now;
        gesture_cell.set(gesture);
        return true;
    }
    if is_scrollable_in_direction(
        panel.scroll_top(),
        panel.scroll_height(),
        panel.client_height(),
        delta,
    ) {
        gesture_cell.set(WheelGesture::default());
        return false;
    }
    gesture.locked_until = 0.0;
    if now - gesture.last_at > WHEEL_GESTURE_GAP_MS
        || gesture.accumulated.signum() != delta.signum()
    {
        gesture.accumulated = 0.0;
    }
    gesture.last_at = now;
    gesture.accumulated += delta;
    if gesture.accumulated.abs() >= WHEEL_THRESHOLD_PX {
        let next = match (state.page.get_untracked(), delta > 0.0) {
            (NavigationPage::Main, true) => NavigationPage::Books,
            (NavigationPage::Books, false) => NavigationPage::Main,
            (page, _) => page,
        };
        if next != state.page.get_untracked() {
            navigate_root_page(state, next, has_focus_within(panel_ref), &navigate);
            gesture.locked_until = now + WHEEL_FLIP_LOCK_MS;
        }
        gesture.accumulated = 0.0;
    }
    gesture_cell.set(gesture);
    true
}

#[cfg(feature = "hydrate")]
fn handle_navigation_key(
    event: &ev::KeyboardEvent,
    state: NavigationState,
    navigate: impl Fn(&str, NavigateOptions),
) {
    if event.default_prevented()
        || event.alt_key()
        || event.ctrl_key()
        || event.meta_key()
        || event.shift_key()
    {
        return;
    }
    let (next, href) = match (state.page.get_untracked(), event.key().as_str()) {
        (NavigationPage::Main, "PageDown" | "ArrowDown") => {
            event.prevent_default();
            navigate_root_page(state, NavigationPage::Books, true, navigate);
            return;
        }
        (NavigationPage::Books, "PageUp" | "ArrowUp") => {
            event.prevent_default();
            navigate_root_page(state, NavigationPage::Main, true, navigate);
            return;
        }
        (NavigationPage::Book, "ArrowLeft") => (NavigationPage::Books, Some("/writing".to_owned())),
        (NavigationPage::Chapter, "ArrowLeft") => {
            let href = state
                .book
                .get_untracked()
                .map(|book| format!("/writing/books/{}", book.slug));
            (NavigationPage::Book, href)
        }
        (NavigationPage::Article, "ArrowLeft") => {
            let location = state
                .article
                .get_untracked()
                .and_then(|post| find_post_location(post.slug));
            match location {
                Some((book, chapter)) => (
                    NavigationPage::Chapter,
                    Some(build_chapter_path(book, chapter)),
                ),
                None => (NavigationPage::Books, Some("/writing".to_owned())),
            }
        }
        _ => return,
    };
    event.prevent_default();
    state.change_page(next, true);
    if let Some(href) = href {
        navigate(&href, NavigateOptions::default());
    }
}

#[cfg(feature = "hydrate")]
fn focus_heading(target: NodeRef<html::H2>) {
    let Some(element) = target.get_untracked() else {
        return;
    };
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    if let Err(source) = element.focus_with_options(&options) {
        web_sys::console::error_2(
            &"Failed to focus reading navigation heading".into(),
            &source,
        );
    }
}

#[cfg(feature = "hydrate")]
fn schedule_heading_focus(
    target: NodeRef<html::H2>,
    page: NavigationPage,
    state: NavigationState,
    pending: StoredValue<RefCell<Option<AnimationFrameRequestHandle>>, LocalStorage>,
) {
    if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
        handle.cancel();
    }
    match request_animation_frame_with_handle(move || {
        pending.with_value(|pending| {
            pending.borrow_mut().take();
        });
        if state.page.get_untracked() == page {
            focus_heading(target);
            state.focus_request.set(None);
        }
    }) {
        Ok(handle) => pending.with_value(|pending| *pending.borrow_mut() = Some(handle)),
        Err(source) => web_sys::console::error_2(
            &"Failed to schedule reading navigation focus".into(),
            &source,
        ),
    }
}

#[component]
fn BooksPage(
    state: NavigationState,
    pathname: Memo<String>,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-books"
            inert=move || state.page.get() != NavigationPage::Books
            aria-hidden=move || if state.page.get() == NavigationPage::Books { "false" } else { "true" }>
            <div class="navigation-page-header">
                <A attr:class="sidebar-link navigation-back" href="/" attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.select_route(None, true);
                        }
                    }>
                    <span aria-hidden="true">"←"</span>{move || locale.get().select("返回主导航", "Back to navigation")}
                </A>
                <h2 node_ref=heading tabindex="-1">{move || locale.get().select("书籍", "Books")}</h2>
            </div>
            <nav class="reading-navigation-books sidebar-nav" aria-label=move || locale.get().select("书籍", "Books")>
                {BOOKS.iter().map(|book| {
                    let href = format!("/writing/books/{}", book.slug);
                    let active_href = href.clone();
                    view! {
                        <A href=href attr:data-sidebar-level=""
                            attr:class=move || if pathname.with(|path| path == &active_href) { "sidebar-link is-active" } else { "sidebar-link" }
                            on:click=move |event| {
                                if is_unmodified_click(&event) {
                                    state.select_route(Some(ReadingRoute::Book(book)), true);
                                }
                            }>
                            <Icon kind=IconKind::Writing/><span lang="zh-CN">{book.title}</span>
                        </A>
                    }
                }).collect_view()}
            </nav>
        </div>
    }
}

#[component]
fn BookPage(
    state: NavigationState,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-book"
            inert=move || state.page.get() != NavigationPage::Book
            aria-hidden=move || if state.page.get() == NavigationPage::Book { "false" } else { "true" }>
            <div class="navigation-page-header">
                <A attr:class="sidebar-link navigation-back" href="/writing" attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.select_route(Some(ReadingRoute::Index), true);
                        }
                    }>
                    <span aria-hidden="true">"←"</span>{move || locale.get().select("返回书籍", "Back to books")}
                </A>
                <h2 node_ref=heading tabindex="-1" lang="zh-CN">
                    {move || state.book.get().map(|book| book.title)}
                </h2>
                <p class="navigation-page-label">{move || locale.get().select("章节", "Chapters")}</p>
            </div>
            <nav class="reading-navigation-chapters sidebar-nav" aria-label=move || locale.get().select("章节", "Chapters")>
                {move || state.book.get().map(|book| {
                    if book.chapters.is_empty() {
                        view! { <p class="reading-navigation-empty">{move || locale.get().select("暂无章节", "No chapters yet")}</p> }.into_any()
                    } else {
                        book.chapters.iter().map(|chapter| {
                            let href = build_chapter_path(book, chapter);
                            view! {
                                <A href=href attr:data-sidebar-level="" attr:class="sidebar-link"
                                    on:click=move |event| {
                                        if is_unmodified_click(&event) {
                                            state.select_route(Some(ReadingRoute::Chapter(book, chapter)), true);
                                        }
                                    }>
                                    <span lang="zh-CN">{chapter.title}</span>
                                </A>
                            }
                        }).collect_view().into_any()
                    }
                })}
            </nav>
        </div>
    }
}

fn render_chapter_articles(
    chapter: &'static Chapter,
    state: NavigationState,
    pathname: Memo<String>,
    locale: RwSignal<Locale>,
) -> AnyView {
    if chapter.post_slugs.is_empty() {
        return view! { <p class="reading-navigation-empty">{move || locale.get().select("暂无文章", "No articles yet")}</p> }.into_any();
    }
    chapter.post_slugs.iter().map(|slug| match find_post(slug) {
        Some(post) => {
            let href = format!("/writing/{}", post.slug);
            let active_href = href.clone();
            view! {
                <A href=href
                    attr:class=move || if pathname.with(|path| path == &active_href) { "sidebar-link is-active" } else { "sidebar-link" }
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.select_route(Some(ReadingRoute::Article(post)), true);
                        }
                    }>
                    <span lang="zh-CN">{post.title}</span>
                </A>
            }.into_any()
        }
        None => view! { <span class="reading-navigation-empty">{format!("Unavailable article: {slug}")}</span> }.into_any(),
    }).collect_view().into_any()
}

#[component]
fn ChapterPage(
    state: NavigationState,
    pathname: Memo<String>,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-chapter"
            inert=move || state.page.get() != NavigationPage::Chapter
            aria-hidden=move || if state.page.get() == NavigationPage::Chapter { "false" } else { "true" }>
            <div class="navigation-page-header">
                <A attr:class="sidebar-link navigation-back"
                    href=move || state.book.get()
                        .map(|book| format!("/writing/books/{}", book.slug))
                        .unwrap_or_else(|| "/writing".to_owned())
                    attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.change_page(NavigationPage::Book, true);
                        }
                    }>
                    <span aria-hidden="true">"←"</span>{move || locale.get().select("返回本书", "Back to book")}
                </A>
                <h2 node_ref=heading tabindex="-1" lang="zh-CN">
                    {move || state.chapter.get().map(|chapter| chapter.title)}
                </h2>
                <p class="navigation-page-label">{move || locale.get().select("本章文章", "Articles in this chapter")}</p>
            </div>
            <nav class="reading-navigation-articles sidebar-nav" aria-label=move || locale.get().select("文章", "Articles")>
                {move || state.chapter.get().map(|chapter| render_chapter_articles(chapter, state, pathname, locale))}
            </nav>
        </div>
    }
}

#[component]
fn ArticlePage(
    state: NavigationState,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-article"
            inert=move || state.page.get() != NavigationPage::Article
            aria-hidden=move || if state.page.get() == NavigationPage::Article { "false" } else { "true" }>
            <div class="navigation-page-header">
                <A attr:class="sidebar-link navigation-back"
                    href=move || state.article.get()
                        .and_then(|post| find_post_location(post.slug))
                        .map(|(book, chapter)| build_chapter_path(book, chapter))
                        .unwrap_or_else(|| "/writing".to_owned())
                    attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            let parent = if state.book.get_untracked().is_some() {
                                NavigationPage::Chapter
                            } else {
                                NavigationPage::Books
                            };
                            state.change_page(parent, true);
                        }
                    }>
                    <span aria-hidden="true">"←"</span>
                    {move || if state.book.get().is_some() {
                        locale.get().select("返回章节", "Back to chapter")
                    } else {
                        locale.get().select("返回书籍", "Back to books")
                    }}
                </A>
                <h2 node_ref=heading tabindex="-1" lang="zh-CN">
                    {move || state.article.get().map(|post| post.title)}
                </h2>
                <p class="navigation-page-label">{move || locale.get().select("本文目录", "On this page")}</p>
            </div>
            {move || state.article.get().map(|post| view! { <SidebarOutline post/> })}
        </div>
    }
}

#[component]
fn MainPage(
    state: NavigationState,
    heading: NodeRef<html::H2>,
    panel: NodeRef<html::Div>,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    view! {
        <div node_ref=panel class="navigation-page navigation-page-main"
            inert=move || state.page.get() != NavigationPage::Main
            aria-hidden=move || if state.page.get() == NavigationPage::Main { "false" } else { "true" }>
            <div class="navigation-page-header">
                <h2 node_ref=heading tabindex="-1">{move || locale.get().select("主导航", "Main navigation")}</h2>
            </div>
            <SidebarNavigation show_writing_link=false/>
            <div class="navigation-entry">
                <A attr:class="sidebar-link" href="/writing" attr:data-sidebar-level=""
                    on:click=move |event| {
                        if is_unmodified_click(&event) {
                            state.select_route(Some(ReadingRoute::Index), true);
                        }
                    }>
                    <Icon kind=IconKind::Writing/>{move || locale.get().select("文章 / 书籍", "Writing / Books")}
                </A>
                <p>{move || locale.get().select("向下滚动浏览书籍", "Scroll down to browse books")}</p>
            </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
fn handle_wheel_input(
    event: &ev::WheelEvent,
    state: NavigationState,
    is_visible: Signal<bool>,
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
    gesture: &Cell<WheelGesture>,
    navigate: impl Fn(&str, NavigateOptions),
) {
    if !is_visible.get_untracked()
        || event.ctrl_key()
        || event.meta_key()
        || event.alt_key()
        || event.shift_key()
    {
        return;
    }
    let panel = panels[state.page.get_untracked().index()];
    let delta = match event.delta_mode() {
        WHEEL_LINE_MODE => event.delta_y() * WHEEL_LINE_PX,
        WHEEL_PAGE_MODE => {
            event.delta_y()
                * f64::from(
                    panel
                        .get_untracked()
                        .map_or(0, |panel| panel.client_height()),
                )
        }
        _ => event.delta_y(),
    };
    if delta.abs() < f64::EPSILON {
        return;
    }
    if handle_navigation_wheel(state, panel, delta, event.time_stamp(), gesture, navigate) {
        event.prevent_default();
    }
}

#[cfg(feature = "hydrate")]
type WheelListener = (web_sys::HtmlElement, Closure<dyn FnMut(ev::WheelEvent)>);

#[cfg(feature = "hydrate")]
fn remove_wheel_listener((element, callback): WheelListener) {
    if let Err(source) =
        element.remove_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())
    {
        web_sys::console::error_2(
            &"Failed to remove reading navigation wheel listener".into(),
            &source,
        );
    }
}

#[cfg(feature = "hydrate")]
fn install_navigation_wheel(
    container: NodeRef<html::Div>,
    state: NavigationState,
    is_visible: Signal<bool>,
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
    navigate: impl Fn(&str, NavigateOptions) + Clone + 'static,
) {
    let listener = StoredValue::new_local(RefCell::new(None::<WheelListener>));
    let gesture = StoredValue::new_local(Cell::new(WheelGesture::default()));
    Effect::new(move |_| {
        if let Some(previous) = listener.with_value(|listener| listener.borrow_mut().take()) {
            remove_wheel_listener(previous);
        }
        let Some(element) = container.get() else {
            return;
        };
        let element: web_sys::HtmlElement = element.into();
        let navigate = navigate.clone();
        let callback = Closure::<dyn FnMut(ev::WheelEvent)>::new(move |event| {
            gesture.with_value(|gesture| {
                handle_wheel_input(&event, state, is_visible, panels, gesture, &navigate)
            });
        });
        let options = web_sys::AddEventListenerOptions::new();
        options.set_passive(false);
        if let Err(source) = element
            .add_event_listener_with_callback_and_add_event_listener_options(
                "wheel",
                callback.as_ref().unchecked_ref(),
                &options,
            )
        {
            web_sys::console::error_2(
                &"Failed to attach reading navigation wheel listener".into(),
                &source,
            );
            return;
        }
        listener.with_value(|listener| *listener.borrow_mut() = Some((element, callback)));
    });
    on_cleanup(move || {
        if let Some(previous) = listener.with_value(|listener| listener.borrow_mut().take()) {
            remove_wheel_listener(previous);
        }
    });
}
#[derive(Clone, Copy)]
struct NavigationNodes {
    headings: [NodeRef<html::H2>; NavigationPage::COUNT],
    panels: [NodeRef<html::Div>; NavigationPage::COUNT],
}

#[cfg(feature = "hydrate")]
fn watch_navigation_updates(
    pathname: Memo<String>,
    initial_path: String,
    reading_request: RwSignal<u64>,
    state: NavigationState,
    is_visible: Signal<bool>,
    nodes: NavigationNodes,
    navigate: impl Fn(&str, NavigateOptions) + 'static,
) {
    Effect::new(move |previous_path: Option<String>| {
        let path = pathname.get();
        if previous_path.as_deref().unwrap_or(initial_path.as_str()) != path {
            let had_sidebar_focus = is_visible.get_untracked()
                && has_focus_within(nodes.panels[state.page.get_untracked().index()]);
            let page = state.select_route(resolve_reading_route(&path), had_sidebar_focus);
            if !had_sidebar_focus && state.focus_request.get_untracked() != Some(page) {
                state.focus_request.set(None);
            }
        }
        path
    });
    Effect::new(move |previous_request: Option<u64>| {
        let request = reading_request.get();
        if previous_request.is_some_and(|previous| previous != request) {
            state.change_page(NavigationPage::Books, true);
            if pathname.get_untracked() != "/writing" {
                navigate("/writing", NavigateOptions::default());
            }
        }
        request
    });
    let pending = StoredValue::new_local(RefCell::new(None::<AnimationFrameRequestHandle>));
    Effect::new(move |_| {
        let target = state.focus_request.get();
        if !is_visible.get() || target.is_none() {
            if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
                handle.cancel();
            }
            return;
        }
        let page = target.unwrap_or(NavigationPage::Main);
        schedule_heading_focus(nodes.headings[page.index()], page, state, pending);
    });
    on_cleanup(move || {
        if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
            handle.cancel();
        }
    });
}
#[component]
pub(crate) fn ReadingNavigation(
    is_hydrated: RwSignal<bool>,
    is_visible: Signal<bool>,
    reading_request: RwSignal<u64>,
) -> impl IntoView {
    let pathname = use_location().pathname;
    let initial_path = pathname.get_untracked();
    let state = NavigationState {
        page: RwSignal::new(NavigationPage::Main),
        book: RwSignal::new(None),
        chapter: RwSignal::new(None),
        article: RwSignal::new(None),
        focus_request: RwSignal::new(None),
    };
    state.select_route(resolve_reading_route(&initial_path), false);
    let nodes = NavigationNodes {
        headings: std::array::from_fn(|_| NodeRef::new()),
        panels: std::array::from_fn(|_| NodeRef::new()),
    };
    #[cfg(feature = "hydrate")]
    watch_navigation_updates(
        pathname,
        initial_path,
        reading_request,
        state,
        is_visible,
        nodes,
        use_navigate(),
    );
    #[cfg(not(feature = "hydrate"))]
    let _ = (initial_path, reading_request);
    let _ = is_hydrated;
    view! {
        <NavigationSurface state pathname is_visible nodes/>
    }
}

#[component]
fn NavigationSurface(
    state: NavigationState,
    pathname: Memo<String>,
    is_visible: Signal<bool>,
    nodes: NavigationNodes,
) -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    let [
        main_heading,
        books_heading,
        book_heading,
        chapter_heading,
        article_heading,
    ] = nodes.headings;
    let [
        main_panel,
        books_panel,
        book_panel,
        chapter_panel,
        article_panel,
    ] = nodes.panels;
    let container = NodeRef::<html::Div>::new();
    #[cfg(feature = "hydrate")]
    let navigate = use_navigate();
    #[cfg(feature = "hydrate")]
    install_navigation_wheel(container, state, is_visible, nodes.panels, navigate.clone());
    #[cfg(not(feature = "hydrate"))]
    let _ = is_visible;
    view! {
        <div node_ref=container class="reading-navigation" role="region"
            aria-label=move || locale.get().select("阅读导航", "Reading navigation")
            on:keydown=move |event| {
                #[cfg(feature = "hydrate")]
                handle_navigation_key(&event, state, &navigate);
                #[cfg(not(feature = "hydrate"))]
                let _ = event;
            }>
            <div class="reading-navigation-track"
                class:page-main=move || state.page.get() == NavigationPage::Main
                class:page-books=move || state.page.get() == NavigationPage::Books
                class:page-book=move || state.page.get() == NavigationPage::Book
                class:page-chapter=move || state.page.get() == NavigationPage::Chapter
                class:page-article=move || state.page.get() == NavigationPage::Article>
                <MainPage state heading=main_heading panel=main_panel/>
                <BooksPage state pathname heading=books_heading panel=books_panel/>
                <BookPage state heading=book_heading panel=book_panel/>
                <ChapterPage state pathname heading=chapter_heading panel=chapter_panel/>
                <ArticlePage state heading=article_heading panel=article_panel/>
            </div>
        </div>
    }
}
