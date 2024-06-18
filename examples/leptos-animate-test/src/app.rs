use crate::animated_swap_page::AnimatedSwapPage;
use crate::dynamics_page::DynamicsPage;
use leptos::*;
use leptos_animate::{
    AnimatedFor, AnimatedLayout, DynamicsAnimation, FadeAnimation, LayoutEntry, LayoutResult,
};
use leptos_meta::*;
use leptos_router::*;
use std::time::Duration;

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
                <Routes>
                    <Route path="" view=AnimatedForPage/>
                    <Route path="/layout" view=AnimatedLayoutPage/>
                    <Route path="/dynamics" view=DynamicsPage/>
                    <Route path="/swap" view=AnimatedSwapPage/>
                    <Route path="/*any" view=NotFound/>
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
            <A href="/dynamics">Dynamics</A>
        </nav>
    }
}

/// Renders the home page of your application.
#[component]
fn AnimatedForPage() -> impl IntoView {
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

    let enter_anim = FadeAnimation::new(Duration::from_millis(500), "ease-out");
    let leave_anim = FadeAnimation::new(Duration::from_millis(500), "ease-out");
    let move_anim = DynamicsAnimation::new(2.0, 0.65, 0.0);

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
                <AnimatedFor each key children animate_size=true enter_anim leave_anim move_anim />
            </div>
        </div>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
enum WindowKind {
    Main,
    Edit,
    EditOptions,
}

#[component]
fn AnimatedLayoutPage() -> impl IntoView {
    let variant = RwSignal::new(WindowKind::Main);

    let set_variant_one = move |_| variant.set(WindowKind::Main);
    let set_variant_two = move |_| variant.set(WindowKind::Edit);
    let set_variant_three = move |_| variant.set(WindowKind::EditOptions);

    let main_view = move || {
        (view! {
            <div class="main-view">
                "Main view"
            </div>
        })
        .into_view()
    };

    let edit_view = move || {
        (view! {
            <div class="edit-view">
                "Edit view"
            </div>
        })
        .into_view()
    };

    let options_view = move || {
        (view! {
            <div class="edit-options-view">
                "Options view"
            </div>
        })
        .into_view()
    };

    let contents = move || {
        let variant = variant.get();
        match variant {
            WindowKind::Main => LayoutResult {
                class: Some("main-mode".into()),
                entries: vec![LayoutEntry {
                    key: WindowKind::Main,
                    view_fn: Box::new(main_view),
                }],
            },
            WindowKind::Edit => LayoutResult {
                class: Some("edit-mode".into()),
                entries: vec![
                    LayoutEntry {
                        key: WindowKind::Edit,
                        view_fn: Box::new(edit_view),
                    },
                    LayoutEntry {
                        key: WindowKind::Main,
                        view_fn: Box::new(main_view),
                    },
                ],
            },
            WindowKind::EditOptions => LayoutResult {
                class: Some("edit-options-mode".into()),
                entries: vec![
                    LayoutEntry {
                        key: WindowKind::EditOptions,
                        view_fn: Box::new(options_view),
                    },
                    LayoutEntry {
                        key: WindowKind::Edit,
                        view_fn: Box::new(edit_view),
                    },
                ],
            },
        }
    };

    view! {
        <div class="main-container">
            <div class="buttons">
                <button on:click=set_variant_one>"Main"</button>
                <button on:click=set_variant_two>"Edit"</button>
                <button on:click=set_variant_three>"Edit + Options"</button>
            </div>
            <AnimatedLayout contents />
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
