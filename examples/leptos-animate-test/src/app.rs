use leptos::*;
use leptos_animate::AnimatedFor;
use leptos_meta::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos-animate-test.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/*any" view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let next_val = StoredValue::new(6);
    let elements = RwSignal::new(vec![1, 2, 3, 4, 5]);

    let on_click = move |_| {
        elements.update(|v| v.push(next_val.get_value()));
        next_val.update_value(|v| *v += 1);
    };

    let each = move || elements.get();

    let key = move |v: &i32| *v;

    let children = move |c: &i32| {
        let c = *c;
        view! {
            <div class="element">{c}</div>
        }
    };

    view! {
        <div class="main-grid">
            <AnimatedFor each key children />
            <button on:click=on_click>"+ Add"</button>
        </div>
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