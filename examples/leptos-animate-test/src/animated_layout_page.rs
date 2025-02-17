use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_animate::{AnimatedLayout, LayoutEntry, LayoutResult};

#[component]
pub fn AnimatedLayoutPage() -> impl IntoView {
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
        .into_any()
    };

    let edit_view = move || {
        (view! {
            <div class="edit-view">
                "Edit view"
            </div>
        })
        .into_any()
    };

    let options_view = move || {
        (view! {
            <div class="edit-options-view">
                "Options view"
            </div>
        })
        .into_any()
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

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
enum WindowKind {
    Main,
    Edit,
    EditOptions,
}
