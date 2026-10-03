use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(super) enum IconKind {
    Home,
    Writing,
    Focus,
    About,
    Sidebar,
    Moon,
    Sun,
    External,
    Close,
}

#[component]
pub(super) fn Icon(kind: IconKind) -> impl IntoView {
    let paths = match kind {
        IconKind::Home => view! { <path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z"/> }.into_any(),
        IconKind::Writing => view! { <><path d="M4 5h16v15H4z"/><path d="M8 9h8M8 13h8M8 17h5"/></> }.into_any(),
        IconKind::Focus => view! { <><circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 1v4m0 14v4M1 12h4m14 0h4"/></> }.into_any(),
        IconKind::About => view! { <><circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10h.01"/></> }.into_any(),
        IconKind::Sidebar => view! { <><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M9 3v18"/></> }.into_any(),
        IconKind::Moon => view! { <path d="M20.3 16.3A8.5 8.5 0 0 1 7.7 3.7 8.5 8.5 0 1 0 20.3 16.3Z"/> }.into_any(),
        IconKind::Sun => view! { <><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></> }.into_any(),
        IconKind::External => view! { <><path d="M14 4h6v6m0-6-9 9"/><path d="M20 14v5a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h5"/></> }.into_any(),
        IconKind::Close => view! { <path d="M5 5l14 14M19 5 5 19"/> }.into_any(),
    };
    view! {
        <svg aria-hidden="true" focusable="false" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            {paths}
        </svg>
    }
}
