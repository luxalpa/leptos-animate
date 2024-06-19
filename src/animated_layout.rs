use leptos::*;

use crate::{
    AnimatedFor, AnyEnterAnimation, AnyLeaveAnimation, AnyMoveAnimation, FadeAnimation,
    SlidingAnimation,
};
use std::hash::Hash;

/// Part of the return value for [`AnimatedLayout`] describing each individual view.
pub struct LayoutEntry<K: Hash + Eq + Clone + 'static> {
    /// The unique key for this view.
    pub key: K,

    /// A function that will be called to create the view.
    pub view_fn: Box<dyn Fn() -> View>,
}

/// The return value for [`AnimatedLayout`], containing the new class being set and the list of
/// elements to render. Only those that aren't already existing (determined by their keys) will be
/// rendered.
pub struct LayoutResult<K: Hash + Eq + Clone + 'static> {
    pub class: Option<Oco<'static, str>>,
    pub entries: Vec<LayoutEntry<K>>,
}

/// Variant of [`AnimatedFor`] / [`AnimatedSwap`] that handles layout-related style changes that
/// need to be applied when the elements change.
///
/// Useful for handling transitions between page layouts, for example when the containers
/// `grid-template-columns`, etc changes. These CSS changes have to happen at the exact right timing
///  - before the elements take their new snapshots but after they took their initial ones.
///
/// Just like with [`AnimatedFor`], these page layouts must not depend on the sizes of the child
/// elements.
///
/// Note that unlike [`AnimatedFor`], this wraps its contents in a top level `<div />`
#[component]
pub fn AnimatedLayout<K, ContentsFn>(
    /// A signal-like function that will return the list of elements to show as well as the new
    /// class to set on the container.
    contents: ContentsFn,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    enter_anim: AnyEnterAnimation,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = FadeAnimation::default().into(), into)]
    leave_anim: AnyLeaveAnimation,

    /// See this prop on [`AnimatedFor`].
    #[prop(default = SlidingAnimation::default().into(), into)]
    move_anim: AnyMoveAnimation,
) -> impl IntoView
where
    K: Hash + Eq + Clone + 'static,
    ContentsFn: Fn() -> LayoutResult<K> + 'static,
{
    let new_class = StoredValue::new(None::<Oco<'static, str>>);
    let class = RwSignal::new(None::<Oco<'static, str>>);

    let each = move || {
        let contents = contents();
        new_class.set_value(contents.class);
        contents.entries
    };

    let key = move |v: &LayoutEntry<K>| v.key.clone();

    let children = move |v: &LayoutEntry<K>| (v.view_fn)();

    let on_after_snapshot = Callback::new(move |_| {
        class.set(new_class.get_value());
    });

    let inner = view! {
        <AnimatedFor
            each
            key
            children
            on_after_snapshot
            animate_size=true
            enter_anim
            move_anim
            leave_anim
        />
    };

    view! {
        <div class=class>
            {inner}
        </div>
    }
}
