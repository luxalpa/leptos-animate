use leptos::*;
use leptos_animate::AnimatedSwap;

#[derive(Clone)]
enum Variant {
    VariantA,
    VariantB,
    VariantC,
}

#[component]
pub fn AnimatedSwapPage() -> impl IntoView {
    let variant = RwSignal::new(Variant::VariantA);

    let content = Signal::derive(move || match variant.get() {
        Variant::VariantA => (view! {
            <div class="var-a">
                "Variant A"
            </div>
        })
        .into_view(),
        Variant::VariantB => (view! {
            <div class="var-b">
                "B"
            </div>
        })
        .into_view(),
        Variant::VariantC => (view! {
            <div class="var-c">
                "Some more things in variant C"
            </div>
        })
        .into_view(),
    });

    let set_variant_a = move |_| variant.set(Variant::VariantA);
    let set_variant_b = move |_| variant.set(Variant::VariantB);
    let set_variant_c = move |_| variant.set(Variant::VariantC);

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
                <AnimatedSwap content />
            </div>
        </div>
    }
}
