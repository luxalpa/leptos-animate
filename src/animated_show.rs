use leptos::*;

use crate::{AnimatedFor, AnyEnterAnimation, AnyLeaveAnimation, FadeAnimation};

#[component]
pub fn AnimatedShow(
    children: ChildrenFn,
    when: Signal<bool>,
    #[prop(default = FadeAnimation::default().into(), into)] enter_anim: AnyEnterAnimation,
    #[prop(default = FadeAnimation::default().into(), into)] leave_anim: AnyLeaveAnimation,
    #[prop(default = false)] appear: bool,
    #[prop(default = false)] handle_margins: bool,
) -> impl IntoView {
    let each = move || {
        if when.get() {
            vec![()]
        } else {
            vec![]
        }
    };

    let children_fn = move |_d: &()| children();

    view! {
        <AnimatedFor each key=|_| 0 children=children_fn
            appear enter_anim leave_anim handle_margins
        />
    }
}
