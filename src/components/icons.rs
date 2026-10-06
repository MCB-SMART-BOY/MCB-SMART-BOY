use leptos::prelude::*;

#[derive(Clone, Copy)]
pub(super) enum IconKind {
    Home,
    Writing,
    Focus,
    About,
    Education,
    Code,
    Work,
    Community,
    Award,
    Sidebar,
    Moon,
    Sun,
    Share,
    Github,
    Bilibili,
    Close,
}

#[component]
pub(super) fn Icon(kind: IconKind) -> impl IntoView {
    let paths = match kind {
        IconKind::Home => view! { <path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1z"/> }.into_any(),
        IconKind::Writing => view! { <><path d="M4 5h16v15H4z"/><path d="M8 9h8M8 13h8M8 17h5"/></> }.into_any(),
        IconKind::Focus => view! { <><circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/><path d="M12 1v4m0 14v4M1 12h4m14 0h4"/></> }.into_any(),
        IconKind::About => view! { <><circle cx="12" cy="12" r="9"/><path d="M12 11v6m0-10h.01"/></> }.into_any(),
        IconKind::Education => view! { <><path d="m2 9 10-5 10 5-10 5zM6 11v6c4 3 8 3 12 0v-6M22 9v7"/></> }.into_any(),
        IconKind::Code => view! { <path d="m7 7-5 5 5 5m10-10 5 5-5 5M14 4l-4 16"/> }.into_any(),
        IconKind::Work => view! { <><rect x="3" y="7" width="18" height="14" rx="2"/><path d="M8 7V4h8v3M3 12c6 4 12 4 18 0M12 12v4"/></> }.into_any(),
        IconKind::Community => view! { <><circle cx="9" cy="8" r="3"/><path d="M2 21v-3a7 7 0 0 1 14 0v3M16 5a3 3 0 0 1 0 6M18 14a5 5 0 0 1 4 5v2"/></> }.into_any(),
        IconKind::Award => view! { <><circle cx="12" cy="8" r="5"/><path d="m8 12-2 10 6-3 6 3-2-10"/></> }.into_any(),
        IconKind::Sidebar => view! { <><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M9 3v18"/></> }.into_any(),
        IconKind::Moon => view! { <path d="M20.3 16.3A8.5 8.5 0 0 1 7.7 3.7 8.5 8.5 0 1 0 20.3 16.3Z"/> }.into_any(),
        IconKind::Sun => view! { <><circle cx="12" cy="12" r="4"/><path d="M12 2v2m0 16v2M4.9 4.9l1.4 1.4m11.4 11.4 1.4 1.4M2 12h2m16 0h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/></> }.into_any(),
        IconKind::Close => view! { <path d="M5 5l14 14M19 5 5 19"/> }.into_any(),
        IconKind::Share => view! { <><path d="M12 16V3m0 0 4 4m-4-4L8 7"/><path d="M5 13v6a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-6"/></> }.into_any(),
        IconKind::Github => view! { <><path d="M9 19c-4.2 1.3-4.2-2-5.9-2.4M15 21v-3.4a3.4 3.4 0 0 0-.9-2.6c3-.3 6.1-1.5 6.1-6.8a5.4 5.4 0 0 0-1.4-3.7c.1-.4.6-1.8-.1-3.6 0 0-1.2-.4-3.8 1.4a13 13 0 0 0-6.8 0C5.5.9 4.3 1.3 4.3 1.3c-.7 1.8-.2 3.2-.1 3.6a5.4 5.4 0 0 0-1.4 3.7c0 5.3 3.1 6.5 6.1 6.8a3.4 3.4 0 0 0-.9 2.6V21"/></> }.into_any(),
        IconKind::Bilibili => view! {
            <>
                <rect x="2.5" y="6" width="19" height="15" rx="3"/>
                <path d="m7 2 2.5 4M17 2l-2.5 4M8.5 12.5v2M15.5 12.5v2M9 18c1.8 1 4.2 1 6 0"/>
            </>
        }.into_any(),
    };
    view! {
        <svg aria-hidden="true" focusable="false" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            {paths}
        </svg>
    }
}
