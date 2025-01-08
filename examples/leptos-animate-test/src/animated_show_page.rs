use std::time::Duration;

use leptos::prelude::*;
use leptos_animate::{AnimatedShow, FadeAnimation};

#[component]
pub fn AnimatedShowPage() -> impl IntoView {
    let show = RwSignal::new(true);

    let toggle = move |_| show.update(|v| *v = !*v);

    let enter_anim = FadeAnimation::new(Duration::from_millis(200), "ease-out");
    let leave_anim = FadeAnimation::new(Duration::from_millis(200), "ease-out");

    view! {
        <div class="main-container animated-show-page">
            <div class="buttons">
                <button on:click=toggle>
                    "Toggle Visibility"
                </button>
            </div>
            <AnimatedShow when=show enter_anim leave_anim>
                <div class="child">
                    "Visible Element"
                </div>
            </AnimatedShow>
        </div>
    }
}
