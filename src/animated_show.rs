use leptos::prelude::*;

use crate::{AnimatedFor, AnyEnterAnimation, AnyLeaveAnimation, FadeAnimation};

/// Animated version of [`<Show />`][leptos::Show] without the fallback.
///
/// This is a variant of [`AnimatedFor`] that only shows a single child or no child.
/// For switching between elements, see [`AnimatedSwap`][crate::AnimatedSwap].
///
/// **Note:** Leptos has a component with the same name that is automatically imported with
/// `use leptos::*` but works differently.
/// Importing this one will shadow the other one.
#[component]
pub fn AnimatedShow(
    /// The child to show / hide.
    children: ChildrenFn,

    /// Whether to show the child or not.
    #[prop(into)]
    when: Signal<bool>,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    enter_anim: AnyEnterAnimation,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    leave_anim: AnyLeaveAnimation,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = false)]
    appear: bool,
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
            appear enter_anim leave_anim
        />
    }
}
