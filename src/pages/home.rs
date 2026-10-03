use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::components::A;

use super::post_list::PostList;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="首页 — MCB / LOG"/>
        <Meta name="description" content="MCB-SMART-BOY 的个人博客演示：关于 Rust、系统和持续探索的笔记。"/>
        <Hero/>
        <Ticker/>
        <RecentPosts/>
        <Closing/>
    }
}

#[component]
fn Hero() -> impl IntoView {
    view! {
        <section class="hero" aria-labelledby="hero-title">
            <div class="hero-copy">
                <div class="eyebrow"><span class="pulse" aria-hidden="true"></span>" PERSONAL SITE "<span class="eyebrow-divider">"/"</span>" RUST · SYSTEMS · IDEAS"</div>
                <h1 id="hero-title">"保持好奇"<span class="hero-comma">"，"</span><br/><span class="hero-outline">"持续构建"</span><span class="hero-period">"."</span></h1>
                <p class="hero-lead">"你好，我是 "<strong>"MCB-SMART-BOY"</strong>"。"<br/>"这里放着关于系统、代码与探索过程的笔记。"</p>
                <div class="hero-actions">
                    <A href="/writing" attr:class="button-primary">"阅读文章 "<span aria-hidden="true">"↗"</span></A>
                    <A href="/about" attr:class="button-text">"了解我 "<span aria-hidden="true">"↗"</span></A>
                </div>
                <div class="hero-coordinate" aria-hidden="true">"01 / AN INTERNET CORNER"<br/>"FOR THE CURIOUS MINDS."</div>
            </div>
            <HeroArt/>
        </section>
    }
}

#[component]
fn HeroArt() -> impl IntoView {
    view! {
        <div class="hero-art" aria-hidden="true">
            <div class="art-top"><span>"FIG. 001 — AN OPEN SYSTEM"</span><span>"◱"</span></div>
            <div class="art-orbit art-orbit-one"></div>
            <div class="art-orbit art-orbit-two"></div>
            <div class="art-orbit art-orbit-three"></div>
            <div class="art-core"><span class="art-core-small">"THE PROCESS"</span><span class="art-core-word">"make"<span>" / "</span>"learn"</span><span class="art-core-cross">"✳"</span></div>
            <span class="art-point art-point-a">"+"</span><span class="art-point art-point-b">"+"</span><span class="art-point art-point-c">"+"</span>
            <div class="art-bottom"><span>"LESS MAGIC."<br/>"MORE UNDERSTANDING."</span><span>"001"<br/>"— 003"</span></div>
        </div>
    }
}

#[component]
fn Ticker() -> impl IntoView {
    view! {
        <div class="ticker" aria-label="关注方向">
            <span>"RUST"</span><span class="ticker-symbol" aria-hidden="true">"✳"</span>
            <span>"SYSTEMS THINKING"</span><span class="ticker-symbol" aria-hidden="true">"✳"</span>
            <span>"AI & SECURITY"</span><span class="ticker-symbol" aria-hidden="true">"✳"</span>
            <span>"OPEN QUESTIONS"</span><span class="ticker-symbol" aria-hidden="true">"✳"</span>
        </div>
    }
}

#[component]
fn RecentPosts() -> impl IntoView {
    view! {
        <section class="section writing-section" aria-labelledby="writing-title">
            <div class="section-top">
                <span class="section-kicker"><span class="square" aria-hidden="true"></span>" 01 / THE LOGBOOK"</span>
                <span class="section-aside">"WORDS, IDEAS & EXPERIMENTS"</span>
            </div>
            <div class="section-heading">
                <div><h2 id="writing-title">"最近的"<span class="accent">"记录."</span></h2><p>"一些关于构建与思考的示例文章。正式发布前，请替换为自己的内容。"</p></div>
                <A href="/writing" attr:class="section-link">"全部文章 "<span aria-hidden="true">"↗"</span></A>
            </div>
            <PostList/>
            <A href="/focus" attr:class="section-link focus-link">"了解关注领域 "<span aria-hidden="true">"↗"</span></A>
        </section>
    }
}

#[component]
fn Closing() -> impl IntoView {
    view! {
        <section class="closing" aria-labelledby="closing-title">
            <div><span class="closing-label">"SOMEWHERE TO START /"</span><h2 id="closing-title">"一起看看"<span class="accent">"下一步？"</span></h2></div>
            <a href="https://github.com/MCB-SMART-BOY" target="_blank" rel="noopener noreferrer">"在 GITHUB 找到我 "<span aria-hidden="true">"↗"</span></a>
        </section>
    }
}
