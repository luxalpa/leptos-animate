use leptos::*;

use crate::{
    AnimatedFor, AnyEnterAnimation, AnyLeaveAnimation, AnyMoveAnimation, FadeAnimation,
    SlidingAnimation,
};
use std::hash::Hash;

pub struct LayoutEntry<K: Hash + Eq + Clone + 'static> {
    pub key: K,
    pub view_fn: Box<dyn Fn() -> View>,
}

pub struct LayoutResult<K: Hash + Eq + Clone + 'static> {
    pub class: Option<Oco<'static, str>>,
    pub entries: Vec<LayoutEntry<K>>,
}

#[component]
pub fn AnimatedLayout<K, ContentsFn>(
    contents: ContentsFn,
    #[prop(default = FadeAnimation::default().into(), into)] enter_anim: AnyEnterAnimation,
    #[prop(default = FadeAnimation::default().into(), into)] leave_anim: AnyLeaveAnimation,
    #[prop(default = SlidingAnimation::default().into(), into)] move_anim: AnyMoveAnimation,
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
        <AnimatedFor each key children on_after_snapshot animate_size=true enter_anim move_anim leave_anim />
    };

    view! {
        <div class=class>
            {inner}
        </div>
    }
}
