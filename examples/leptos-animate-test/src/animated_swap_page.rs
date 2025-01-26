use std::time::Duration;

use leptos::prelude::*;
use leptos_animate::{AnimatedSwap, FadeAnimation, LayoutEntry, SizeTransition, SlidingAnimation};

#[derive(Clone, Hash, PartialEq, Eq)]
enum Variant {
    VariantA,
    VariantB,
    VariantC,
}

#[component]
pub fn AnimatedSwapPage() -> impl IntoView {
    let variant = RwSignal::new(Variant::VariantA);

    let contents = move || {
        let variant = variant.get();
        match variant {
            Variant::VariantA => LayoutEntry {
                key: Variant::VariantA,
                view_fn: Box::new(|| {
                    (view! {
                        <div class="var-a">
                            "Variant A"
                        </div>
                    })
                    .into_any()
                }),
            },
            Variant::VariantB => LayoutEntry {
                key: Variant::VariantB,
                view_fn: Box::new(|| {
                    (view! {
                        <div class="var-b">
                            "Variant B"
                        </div>
                    })
                    .into_any()
                }),
            },
            Variant::VariantC => LayoutEntry {
                key: Variant::VariantC,
                view_fn: Box::new(|| {
                    (view! {
                        <div class="var-c">
                            "Variant C"
                        </div>
                    })
                    .into_any()
                }),
            },
        }
    };

    let set_variant_a = move |_| variant.set(Variant::VariantA);
    let set_variant_b = move |_| variant.set(Variant::VariantB);
    let set_variant_c = move |_| variant.set(Variant::VariantC);

    let resize_anim = SlidingAnimation::new(Duration::from_millis(200), "ease-out");
    let enter_anim = FadeAnimation::new(Duration::from_millis(200), "ease-out");
    let leave_anim = FadeAnimation::new(Duration::from_millis(200), "ease-out");

    view! {
        <div class="main-container animated-swap-page">
            <div class="buttons">
                <button on:click=set_variant_a>
                    "Variant A"
                </button>
                <button on:click=set_variant_b>
                    "Variant B"
                </button>
                <button on:click=set_variant_c>
                    "Variant C"
                </button>
            </div>
            <div class="content">
                <SizeTransition resize_anim>
                    <AnimatedSwap contents enter_anim leave_anim />
                </SizeTransition>
            </div>
        </div>
    }
}
