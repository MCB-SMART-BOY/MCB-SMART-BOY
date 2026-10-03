use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

#[component]
pub fn NotFoundPage() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(options) = use_context::<leptos_axum::ResponseOptions>() {
        options.set_status(axum::http::StatusCode::NOT_FOUND);
    }

    view! {
        <Title text="页面未找到 — MCB / LOG"/>
        <Meta name="description" content="这个页面不存在。返回博客首页继续阅读。"/>
        <section class="missing-page">
            <span class="section-kicker"><span class="square" aria-hidden="true"></span>" ERROR / 404"</span>
            <h1>"这里暂时"<span class="accent">"没有信号."</span></h1>
            <p>"你要找的页面不存在，或者地址已经改变。"</p>
            <A href="/" attr:class="button-primary">"返回首页 "<span aria-hidden="true">"↗"</span></A>
        </section>
    }
}
