use crate::{AnimatedFor, AnyEnterAnimation, AnyLeaveAnimation, FadeAnimation};
use leptos::*;

/// Animated transition between views.
#[component]
pub fn AnimatedSwap(
    /// The view to show.
    content: Signal<View>,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = false)]
    appear: bool,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = false)]
    handle_margins: bool,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    enter_anim: AnyEnterAnimation,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    leave_anim: AnyLeaveAnimation,
) -> impl IntoView {
    let key = StoredValue::new(0);

    let element = Memo::new(move |_| {
        let k = (key.get_value() + 1) % 100;
        key.set_value(k);
        content.get()
    });

    let each = move || {
        element.track();
        [key.get_value()]
    };

    let children_fn = move |_: &i32| element.get();

    view! {
        <AnimatedFor
            each
            key=move |k| *k
            children=children_fn
            appear
            animate_size=true
            enter_anim
            leave_anim
            handle_margins
        />
    }
}
