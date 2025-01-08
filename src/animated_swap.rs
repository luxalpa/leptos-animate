use crate::{AnimatedFor, AnyEnterAnimation, AnyLeaveAnimation, FadeAnimation, LayoutEntry};
use leptos::prelude::*;
use std::hash::Hash;

/// Animated transition between views.
#[component]
pub fn AnimatedSwap<K, ContentsFn>(
    /// The view to show.
    contents: ContentsFn,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = false)]
    appear: bool,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    enter_anim: AnyEnterAnimation,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    leave_anim: AnyLeaveAnimation,
) -> impl IntoView
where
    K: Hash + Eq + Clone + 'static + Send + Sync,
    ContentsFn: Fn() -> Vec<LayoutEntry<K>> + 'static + Send + Sync,
{
    let key = move |v: &LayoutEntry<K>| v.key.clone();

    let children = move |v: &LayoutEntry<K>| (v.view_fn)();

    view! {
        <AnimatedFor
            each=contents
            key
            children
            appear
            animate_size=true
            enter_anim
            leave_anim
        />
    }
}
