use leptos::prelude::*;
use leptos_animate::{AnimatedFor, DynamicsAnimation, FadeAnimation};
use std::time::Duration;

#[component]
pub fn AnimatedForPage() -> impl IntoView {
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
