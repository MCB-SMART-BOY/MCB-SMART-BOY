use crate::locale::Locale;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    let locale =
        use_context::<RwSignal<Locale>>().unwrap_or_else(|| RwSignal::new(Locale::default()));
    #[cfg(feature = "ssr")]
    if let Some(options) = use_context::<leptos_axum::ResponseOptions>() {
        options.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <Title text=move || format!("{} — MCB / LOG", locale.get().not_found())/>
        <Meta name="description" content=move || locale.get().select(
            "这个页面不存在。返回博客首页继续阅读。",
            "This page does not exist. Return to the blog homepage to keep reading.",
        )/>
        <section class="missing-page">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" ERROR / 404"</span>
            <h1>{move || locale.get().select("这里暂时", "No signal")}<span class="accent">{move || locale.get().select("没有信号.", " here.")}</span></h1>
            <p>{move || locale.get().select("你要找的页面不存在，或者地址已经改变。", "The page you were looking for does not exist, or its address has changed.")}</p>
            <A href="/" attr:class="button-primary">{move || locale.get().select("返回首页 ", "Back home ")}<span aria-hidden="true">"↗"</span></A>
        </section>
    }
}
