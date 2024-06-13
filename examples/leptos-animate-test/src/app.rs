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
    let next_key = StoredValue::new(6);
    let elements = RwSignal::new(vec![1, 2, 3, 4, 5]);

    let get_next_key = move || {
        let v = next_key.get_value();
        next_key.update_value(|v| *v += 1);
        v
    };

    let add_one = move |_| {
        elements.update(|v| v.push(get_next_key()));
    };

    let remove_and_add = move |_| {
        elements.update(|v| {
            v.remove(0);
            v.push(get_next_key());
        })
    };

    let shift = move |_| {
        elements.update(|v| {
            v.insert(0, get_next_key());
        })
    };

    let reset = move |_| {
        elements.update(|v| {
            v.clear();
            next_key.update_value(|v| *v = 6);
            *v = vec![1, 2, 3, 4, 5];
        });
    };

    let remove_two = move |_| {
        elements.update(|v| {
            v.pop();
            v.pop();
        })
    };

    let each = move || elements.get();

    let key = move |v: &i32| *v;

    let children = move |c: &i32| {
        let c = *c;

        let remove_click = move |_| {
            elements.update(|v| v.retain(|&x| x != c));
        };

        view! {
            <button class="element" on:click=remove_click>{c}</button>
        }
    };

    view! {
        <div class="main-container">
            <div class="buttons">
                <button on:click=add_one>"+ Add"</button>
                <button on:click=remove_and_add>"Remove and Add"</button>
                <button on:click=shift>"Insert first"</button>
                <button on:click=remove_two>"Remove 2"</button>
                <button on:click=reset>"Reset"</button>
            </div>
            <div class="main-grid">
                <AnimatedFor each key children animate_size=true />
            </div>
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
