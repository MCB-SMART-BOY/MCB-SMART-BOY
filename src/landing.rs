use leptos::prelude::*;

use crate::locale::Locale;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum LandingSection {
    Home,
    About,
    Focus,
    Projects,
    Experience,
    Background,
    Community,
    Recognition,
    Notes,
}

impl LandingSection {
    pub(crate) const ALL: [Self; 9] = [
        Self::Home,
        Self::About,
        Self::Focus,
        Self::Projects,
        Self::Experience,
        Self::Background,
        Self::Community,
        Self::Recognition,
        Self::Notes,
    ];

    pub(crate) fn iter_navigation() -> impl Iterator<Item = Self> {
        Self::ALL
            .into_iter()
            .filter(|section| *section != Self::Recognition)
    }

    pub(crate) fn href(self) -> &'static str {
        match self {
            Self::Home => "/#home",
            Self::Focus => "/#focus",
            Self::About => "/#about",
            Self::Background => "/#background",
            Self::Projects => "/#open-source",
            Self::Experience => "/#experience",
            Self::Community => "/#codeflow",
            Self::Recognition => "/#recognition",
            Self::Notes => "/#notes",
        }
    }

    pub(crate) fn id(self) -> &'static str {
        self.href().trim_start_matches("/#")
    }

    pub(crate) fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Home => locale.home(),
            Self::Focus => locale.focus(),
            Self::About => locale.about(),
            Self::Background => locale.select("教育与技能", "Education & skills"),
            Self::Projects => locale.select("项目与开源", "Projects & open source"),
            Self::Experience => locale.select("工程经历", "Engineering experience"),
            Self::Community => locale.select("CodeFlow 社团", "CodeFlow community"),
            Self::Recognition => locale.select("荣誉与交流", "Awards & exchanges"),
            Self::Notes => locale.select("示例文章", "Sample writing"),
        }
    }
}

pub(crate) fn use_landing_section() -> RwSignal<LandingSection> {
    use_context::<RwSignal<LandingSection>>().unwrap_or_else(|| RwSignal::new(LandingSection::Home))
}

#[cfg(feature = "hydrate")]
mod tracking {
    use super::LandingSection;
    use crate::landing_motion::SectionMotion;
    use leptos::prelude::*;
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, closure::Closure};

    const TRACKED_EVENTS: [&str; 4] = ["scroll", "resize", "hashchange", "popstate"];
    // Keep the activation line aligned with the sections' CSS scroll-margin-top.
    const SECTION_CLEARANCE_PX: f64 = 16.0;
    const SCROLL_POSITION_TOLERANCE_PX: f64 = 1.0;
    type TrackerSlot = Rc<RefCell<Option<SectionTracker>>>;

    struct SectionTracker {
        sections: [Option<web_sys::Element>; LandingSection::ALL.len()],
        motions: [Option<SectionMotion>; LandingSection::ALL.len()],
        motion_preference: Option<web_sys::MediaQueryList>,
        scroll_root: web_sys::Element,
        topbar: Option<web_sys::Element>,
        window: web_sys::Window,
        listener: Closure<dyn FnMut(web_sys::Event)>,
        observer: Option<web_sys::ResizeObserver>,
        resize_callback: Option<Closure<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>>,
        pending: Option<AnimationFrameRequestHandle>,
    }

    fn log_browser_error(operation: &str, source: &wasm_bindgen::JsValue) {
        web_sys::console::error_2(&operation.into(), source);
    }

    fn clear_tracker(slot: &TrackerSlot) {
        let Some(mut tracker) = slot.borrow_mut().take() else {
            return;
        };
        if let Some(handle) = tracker.pending.take() {
            handle.cancel();
        }
        if let Some(observer) = tracker.observer.take() {
            observer.disconnect();
        }
        if let Some(query) = &tracker.motion_preference
            && let Err(source) = query.remove_event_listener_with_callback(
                "change",
                tracker.listener.as_ref().unchecked_ref(),
            )
        {
            log_browser_error(
                "Failed to detach landing motion preference listener",
                &source,
            );
        }
        for name in TRACKED_EVENTS {
            if let Err(source) = tracker.window.remove_event_listener_with_callback(
                name,
                tracker.listener.as_ref().unchecked_ref(),
            ) {
                log_browser_error(
                    &format!("Failed to detach landing {name} listener"),
                    &source,
                );
            }
        }
    }

    fn select_visible_section(
        tracker: &SectionTracker,
        rects: &[Option<web_sys::DomRect>; LandingSection::ALL.len()],
        topbar_bottom: f64,
    ) -> LandingSection {
        let scroll_top = tracker.scroll_root.scroll_top();
        let remaining =
            tracker.scroll_root.scroll_height() - tracker.scroll_root.client_height() - scroll_top;
        if scroll_top > 0 && f64::from(remaining) <= SCROLL_POSITION_TOLERANCE_PX {
            return LandingSection::ALL
                .last()
                .copied()
                .unwrap_or(LandingSection::Home);
        }
        let threshold = topbar_bottom + SECTION_CLEARANCE_PX + SCROLL_POSITION_TOLERANCE_PX;
        let mut selected = LandingSection::Home;
        for (section, rect) in LandingSection::ALL.into_iter().zip(rects) {
            if let Some(rect) = rect
                && rect.top() <= threshold
            {
                selected = section;
            }
        }
        selected
    }

    fn sample_landing(tracker: &mut SectionTracker) -> LandingSection {
        let topbar_bottom = tracker
            .topbar
            .as_ref()
            .map(|topbar| topbar.get_bounding_client_rect().bottom())
            .unwrap_or_default();
        let viewport_height = f64::from(tracker.scroll_root.client_height()) - topbar_bottom;
        let rects = tracker.sections.each_ref().map(|section| {
            section
                .as_ref()
                .map(web_sys::Element::get_bounding_client_rect)
        });
        let selected = select_visible_section(tracker, &rects, topbar_bottom);
        let should_animate = tracker
            .motion_preference
            .as_ref()
            .is_some_and(|query| !query.matches());
        // Read all stable anchors before writing transforms to their inner content.
        for (motion, rect) in tracker.motions.iter_mut().zip(&rects) {
            if let (Some(motion), Some(rect)) = (motion, rect) {
                motion.update(rect, topbar_bottom, viewport_height, should_animate);
            }
        }
        selected
    }

    fn schedule_sample(slot: &TrackerSlot, active: RwSignal<LandingSection>) {
        if slot
            .borrow()
            .as_ref()
            .is_none_or(|tracker| tracker.pending.is_some())
        {
            return;
        }
        let frame_slot = Rc::clone(slot);
        match request_animation_frame_with_handle(move || {
            let selected = frame_slot.borrow_mut().as_mut().map(|tracker| {
                tracker.pending.take();
                sample_landing(tracker)
            });
            if let Some(selected) = selected.filter(|selected| active.get_untracked() != *selected)
            {
                active.set(selected);
            }
        }) {
            Ok(handle) => {
                if let Some(tracker) = slot.borrow_mut().as_mut() {
                    tracker.pending = Some(handle);
                } else {
                    handle.cancel();
                }
            }
            Err(source) => log_browser_error("Failed to schedule landing section sample", &source),
        }
    }

    fn observe_layout(
        slot: &TrackerSlot,
        active: RwSignal<LandingSection>,
        main: web_sys::Element,
    ) {
        let callback_slot = Rc::clone(slot);
        let callback =
            Closure::<dyn FnMut(js_sys::Array, web_sys::ResizeObserver)>::new(move |_, _| {
                schedule_sample(&callback_slot, active);
            });
        match web_sys::ResizeObserver::new(callback.as_ref().unchecked_ref()) {
            Ok(observer) => {
                observer.observe(&main);
                if let Some(tracker) = slot.borrow_mut().as_mut() {
                    for section in tracker.sections.iter().flatten() {
                        observer.observe(section);
                    }
                    tracker.observer = Some(observer);
                    tracker.resize_callback = Some(callback);
                } else {
                    observer.disconnect();
                }
            }
            Err(source) => log_browser_error("Failed to observe landing layout changes", &source),
        }
    }

    fn bind_motion_preference(
        window: &web_sys::Window,
        listener: &Closure<dyn FnMut(web_sys::Event)>,
    ) -> Option<web_sys::MediaQueryList> {
        let bind = || -> Result<_, wasm_bindgen::JsValue> {
            let query = window.match_media("(prefers-reduced-motion: reduce)")?;
            if let Some(query) = &query {
                query.add_event_listener_with_callback(
                    "change",
                    listener.as_ref().unchecked_ref(),
                )?;
            }
            Ok(query)
        };
        match bind() {
            Ok(query) => query,
            Err(source) => {
                log_browser_error("Failed to observe landing motion preference", &source);
                None
            }
        }
    }

    fn find_section_elements(
        document: &web_sys::Document,
    ) -> Option<[Option<web_sys::Element>; LandingSection::ALL.len()]> {
        let sections = LandingSection::ALL.map(|section| document.get_element_by_id(section.id()));
        if let Some(index) = sections.iter().position(Option::is_none) {
            web_sys::console::error_2(
                &"Cannot track landing navigation: missing section".into(),
                &LandingSection::ALL[index].href().into(),
            );
            return None;
        }
        Some(sections)
    }

    fn install_tracker(slot: &TrackerSlot, active: RwSignal<LandingSection>) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(document) = window.document() else {
            return;
        };
        let Some(main) = document.get_element_by_id("main") else {
            return;
        };
        let Some(scroll_root) = document.document_element() else {
            return;
        };
        let Some(sections) = find_section_elements(&document) else {
            return;
        };
        let listener_slot = Rc::clone(slot);
        let listener = Closure::<dyn FnMut(web_sys::Event)>::new(move |_| {
            schedule_sample(&listener_slot, active);
        });
        attach_tracker_events(&window, &listener);
        let topbar = match document.query_selector(".site-topbar") {
            Ok(topbar) => topbar,
            Err(source) => {
                log_browser_error("Failed to locate topbar for landing tracker", &source);
                None
            }
        };
        let motions = std::array::from_fn(|index| {
            sections[index]
                .as_ref()
                .and_then(|element| SectionMotion::bind(LandingSection::ALL[index], element))
        });
        let motion_preference = bind_motion_preference(&window, &listener);
        *slot.borrow_mut() = Some(SectionTracker {
            sections,
            motions,
            motion_preference,
            scroll_root,
            topbar,
            window,
            listener,
            observer: None,
            resize_callback: None,
            pending: None,
        });
        observe_layout(slot, active, main);
        schedule_sample(slot, active);
    }

    fn attach_tracker_events(
        window: &web_sys::Window,
        listener: &Closure<dyn FnMut(web_sys::Event)>,
    ) {
        for name in TRACKED_EVENTS {
            if let Err(source) =
                window.add_event_listener_with_callback(name, listener.as_ref().unchecked_ref())
            {
                log_browser_error(
                    &format!("Failed to attach landing {name} listener"),
                    &source,
                );
            }
        }
    }

    pub(super) fn watch_landing(pathname: Memo<String>, active: RwSignal<LandingSection>) {
        let slot: TrackerSlot = Rc::new(RefCell::new(None));
        let pending = StoredValue::new_local(RefCell::new(None::<AnimationFrameRequestHandle>));
        let route_slot = Rc::clone(&slot);
        Effect::new(move |_| {
            let is_root = pathname.get() == "/";
            clear_tracker(&route_slot);
            if let Some(handle) = pending.with_value(|pending| pending.borrow_mut().take()) {
                handle.cancel();
            }
            active.set(LandingSection::Home);
            if !is_root {
                return;
            }
            let next_slot = Rc::clone(&route_slot);
            match request_animation_frame_with_handle(move || {
                pending.with_value(|pending| pending.borrow_mut().take());
                install_tracker(&next_slot, active);
            }) {
                Ok(handle) => pending.with_value(|pending| *pending.borrow_mut() = Some(handle)),
                Err(source) => log_browser_error("Failed to bind landing section tracker", &source),
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
}

pub(crate) fn provide_landing_context(pathname: Memo<String>) {
    let active = RwSignal::new(LandingSection::Home);
    provide_context(active);
    #[cfg(feature = "hydrate")]
    tracking::watch_landing(pathname, active);
    #[cfg(not(feature = "hydrate"))]
    let _ = pathname;
}
