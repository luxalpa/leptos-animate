use crate::animated_for_page::AnimatedForPage;
use crate::animated_layout_page::AnimatedLayoutPage;
use crate::animated_show_page::AnimatedShowPage;
use crate::animated_swap_page::AnimatedSwapPage;
use crate::dynamics_page::DynamicsPage;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::components::{Route, Router, Routes, A};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/leptos-animate-test.css"/>

        <Title text="Leptos Animate"/>

        <Router>
            <main>
                <Navigation/>
                <Routes fallback=move || "Not found.">
                    <Route path=path!("/") view=AnimatedForPage/>
                    <Route path=path!("/layout") view=AnimatedLayoutPage/>
                    <Route path=path!("/dynamics") view=DynamicsPage/>
                    <Route path=path!("/swap") view=AnimatedSwapPage/>
                    <Route path=path!("/show") view=AnimatedShowPage/>
                    <Route path=path!("/*any") view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Navigation() -> impl IntoView {
    view! {
        <nav>
            <A href="/">AnimatedFor</A>
            <A href="/layout">AnimatedLayout</A>
            <A href="/swap">AnimatedSwap</A>
            <A href="/show">AnimatedShow</A>
            <A href="/dynamics">Dynamics</A>
        </nav>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
