use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::{EnterAnimation, FadeAnimation, LeaveAnimation, MoveAnimation, SlidingAnimation};
use indexmap::IndexMap;
use leptos::leptos_dom::is_server;
use leptos::*;
use wasm_bindgen::closure::Closure;
use web_sys::js_sys;
use web_sys::js_sys::Array;
use web_sys::{Animation, FillMode};

use crate::position::{Extent, Position};

struct ItemMeta {
    el: Option<web_sys::HtmlElement>,
    scope: Disposer,
    cur_anim: Option<Animation>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MoveAnimKeyframe {
    transform_origin: String,
    transform: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<String>,
}

// wrapper because the Animation API is unstable and that causes some problems with cranelift.
pub fn animate(
    el: &web_sys::HtmlElement,
    keyframes: Option<&js_sys::Object>,
    duration: &::wasm_bindgen::JsValue,
    fill_mode: FillMode,
    easing: Option<impl AsRef<str>>,
) -> Animation {
    #[cfg(not(feature = "ssr"))]
    {
        use web_sys::KeyframeAnimationOptions;
        let mut options = KeyframeAnimationOptions::new();

        options.duration(duration).fill(fill_mode);

        if let Some(easing) = easing {
            options.easing(easing.as_ref());
        }

        el.animate_with_keyframe_animation_options(keyframes, &options)
    }
    #[cfg(feature = "ssr")]
    {
        _ = el;
        _ = keyframes;
        _ = duration;
        _ = fill_mode;
        _ = easing;
        unimplemented!("Animation API can't be run on the server")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ElementSnapshot {
    position: Position,
    extent: Extent,
}

trait EnterAnimationHandler {
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation;
}

impl<T: EnterAnimation> EnterAnimationHandler for T {
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation {
        let r = self.enter();

        let arr: Array = r
            .keyframes
            .into_iter()
            .map(|v| serde_wasm_bindgen::to_value(&v).unwrap())
            .collect();

        animate(
            &el,
            Some(&arr.into()),
            &(r.duration.as_secs_f64() * 1000.0).into(),
            FillMode::None,
            r.timing_fn.as_ref().map(|v| v.as_str()),
        )
    }
}

#[derive(Clone)]
pub struct AnyEnterAnimation {
    anim: Rc<dyn EnterAnimationHandler>,
}

impl<T: EnterAnimationHandler + 'static> From<T> for AnyEnterAnimation {
    fn from(v: T) -> Self {
        AnyEnterAnimation { anim: Rc::new(v) }
    }
}

pub trait LeaveAnimationHandler {
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation;
}

impl<T: LeaveAnimation> LeaveAnimationHandler for T {
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation {
        let r = self.leave();

        let arr: Array = r
            .keyframes
            .into_iter()
            .map(|v| serde_wasm_bindgen::to_value(&v).unwrap())
            .collect();

        animate(
            &el,
            Some(&arr.into()),
            &(r.duration.as_secs_f64() * 1000.0).into(),
            FillMode::None,
            r.timing_fn.as_ref().map(|v| v.as_str()),
        )
    }
}

pub struct AnyLeaveAnimation {
    anim: Rc<dyn LeaveAnimationHandler>,
}

impl<T: LeaveAnimationHandler + 'static> From<T> for AnyLeaveAnimation {
    fn from(v: T) -> Self {
        AnyLeaveAnimation { anim: Rc::new(v) }
    }
}

pub trait MoveAnimationHandler {
    fn animate(
        &self,
        el: &web_sys::HtmlElement,
        prev_snapshot: ElementSnapshot,
        new_snapshot: ElementSnapshot,
        animate_size: bool,
    ) -> Animation;
}

impl<T: MoveAnimation> MoveAnimationHandler for T {
    fn animate(
        &self,
        el: &web_sys::HtmlElement,
        prev_snapshot: ElementSnapshot,
        new_snapshot: ElementSnapshot,
        animate_size: bool,
    ) -> Animation {
        let r = self.animate(prev_snapshot, new_snapshot);

        let diff = prev_snapshot.position - new_snapshot.position;

        let arr: Array = [
            serde_wasm_bindgen::to_value(&MoveAnimKeyframe {
                transform_origin: "top left".to_string(),
                transform: format!("translate({}px, {}px)", diff.x, diff.y),
                width: animate_size.then(|| format!("{}px", prev_snapshot.extent.width)),
                height: animate_size.then(|| format!("{}px", prev_snapshot.extent.height)),
            })
            .unwrap(),
            serde_wasm_bindgen::to_value(&MoveAnimKeyframe {
                transform_origin: "top left".to_string(),
                transform: "none".to_string(),
                width: animate_size.then(|| format!("{}px", new_snapshot.extent.width)),
                height: animate_size.then(|| format!("{}px", new_snapshot.extent.height)),
            })
            .unwrap(),
        ]
        .into_iter()
        .collect();

        animate(
            &el,
            Some(&arr.into()),
            &(r.duration.as_secs_f64() * 1000.0).into(),
            FillMode::None,
            r.timing_fn.as_ref().map(|v| v.as_str()),
        )
    }
}

pub struct AnyMoveAnimation {
    anim: Rc<dyn MoveAnimationHandler>,
}

impl<T: MoveAnimationHandler + 'static> From<T> for AnyMoveAnimation {
    fn from(v: T) -> Self {
        AnyMoveAnimation { anim: Rc::new(v) }
    }
}

#[component]
pub fn AnimatedFor<IF, I, T, EF, N, KF, K>(
    each: IF,
    key: KF,
    children: EF,
    #[prop(optional)] on_leave_start: Option<Callback<(web_sys::HtmlElement, Position)>>,
    #[prop(optional)] on_enter_start: Option<Callback<web_sys::HtmlElement>>,
    #[prop(optional)] on_after_snapshot: Option<Callback<()>>,
    #[prop(default = false)] appear: bool,
    #[prop(default = false)] animate_size: bool,
    #[prop(default = false)] handle_margins: bool,
    #[prop(default = FadeAnimation::default().into(), into)] enter_anim: AnyEnterAnimation,
    #[prop(default = FadeAnimation::default().into(), into)] leave_anim: AnyLeaveAnimation,
    #[prop(default = SlidingAnimation::default().into(), into)] move_anim: AnyMoveAnimation,
) -> impl IntoView
where
    IF: Fn() -> I + 'static,
    I: IntoIterator<Item = T>,
    EF: Fn(&T) -> N + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + 'static,
    K: Eq + Hash + Clone + 'static,
    T: 'static,
{
    let alive_items = RwSignal::new(IndexMap::<K, T>::new());
    let leaving_items = RwSignal::new(IndexMap::<K, T>::new());
    let key_fn = StoredValue::new(key);
    let alive_items_meta = StoredValue::new(HashMap::<K, ItemMeta>::new());
    let enter_anim = StoredValue::new(enter_anim);
    let leave_anim = StoredValue::new(leave_anim);
    let move_anim = StoredValue::new(move_anim);

    create_isomorphic_effect(move |prev| {
        let new_items = each()
            .into_iter()
            .map(|i| (key_fn.with_value(|k| k(&i)), i))
            .collect::<IndexMap<_, _>>();

        // Get initial snapshots of all elements
        let snapshots = alive_items_meta.with_value(|alive_items_meta| {
            alive_items_meta
                .iter()
                .map(|(k, meta)| {
                    (k.clone(), {
                        if is_server() {
                            ElementSnapshot::default()
                        } else {
                            get_el_snapshot(
                                &meta.el.as_ref().expect("el always exists on the client"),
                                animate_size,
                                handle_margins,
                            )
                        }
                    })
                })
                .collect::<HashMap<_, _>>()
        });

        // Items that are re-added during the animation while they are still leaving must be removed from the leaving_items list and will then be treated as new elements (Their scope already got disposed, so there's no way to resurrect them).
        for k in new_items.keys() {
            if leaving_items.with_untracked(|leaving_items| leaving_items.contains_key(k)) {
                leaving_items.update(|leaving_items| {
                    leaving_items.swap_remove(k);
                });
            }
        }

        // Callback trigger for CSS changes to be applied after snapshots
        if let Some(on_after_snapshot) = on_after_snapshot {
            on_after_snapshot(());
        }

        // Update alive items and trigger exit-animations
        batch({
            let snapshots = &snapshots;
            move || {
                alive_items.update(move |alive_items| {
                    let items_to_remove = alive_items
                        .drain(..)
                        .filter(|(k, _)| !new_items.contains_key(k))
                        .collect::<Vec<_>>();

                    alive_items_meta.update_value(|alive_items_meta| {
                        for (k, _) in items_to_remove.iter() {
                            let Some(ItemMeta {
                                el,
                                scope,
                                cur_anim,
                            }) = alive_items_meta.remove(k)
                            else {
                                continue;
                            };

                            drop(scope);

                            if is_server() {
                                return;
                            }

                            let el = el.expect("el always exists on the client");

                            let snapshot = snapshots.get(k).unwrap();

                            if let Some(on_leave_start) = on_leave_start {
                                on_leave_start((el.clone(), snapshot.position));
                            }

                            let extent = if animate_size {
                                snapshot.extent
                            } else {
                                Extent {
                                    width: el.offset_width() as f64,
                                    height: el.offset_height() as f64,
                                }
                            };

                            if let Some(cur_anim) = cur_anim {
                                cur_anim.cancel();
                            }

                            let style = el.style();
                            style.set_property("position", "absolute").unwrap();
                            style
                                .set_property("top", &format!("{}px", snapshot.position.y))
                                .unwrap();
                            style
                                .set_property("left", &format!("{}px", snapshot.position.x))
                                .unwrap();

                            style
                                .set_property("width", &format!("{}px", extent.width))
                                .unwrap();

                            style
                                .set_property("height", &format!("{}px", extent.height))
                                .unwrap();

                            let anim =
                                leave_anim.with_value(|leave_anim| leave_anim.anim.animate(&el));

                            // Remove leaving elements after their exit-animation
                            let closure = Closure::<dyn Fn(web_sys::Event)>::new({
                                let k = k.clone();
                                move |_| {
                                    leaving_items.try_update(|leaving_items| {
                                        leaving_items.swap_remove(&k);
                                    });
                                }
                            })
                            .into_js_value();

                            anim.set_onfinish(Some(&closure.into()));
                        }
                    });

                    leaving_items.update(move |leaving_items| {
                        leaving_items.extend(items_to_remove);
                    });
                    alive_items.extend(new_items);
                });
            }
        });

        // Wait for the children to be created so that we get element refs for enter-animation
        queue_microtask(move || {
            if is_server() {
                return;
            }
            if prev.is_none() && !appear {
                return;
            }
            alive_items_meta.update_value(|items| {
                for (k, meta) in items.iter_mut() {
                    let el = meta.el.clone().expect("el always exists on the client");
                    let Some(&prev_snapshot) = snapshots.get(k) else {
                        // Enter-animation

                        if let Some(on_enter_start) = on_enter_start {
                            on_enter_start(el.clone());
                        }

                        meta.cur_anim.take().map(|cur_anim| cur_anim.cancel());

                        meta.cur_anim =
                            Some(enter_anim.with_value(|enter_anim| enter_anim.anim.animate(&el)));

                        continue;
                    };

                    // Move-animation

                    meta.cur_anim.take().map(|cur_anim| cur_anim.cancel());

                    let new_snapshot = get_el_snapshot(&el, animate_size, handle_margins);

                    if prev_snapshot == new_snapshot {
                        continue;
                    }

                    meta.cur_anim = Some(move_anim.with_value(|move_anim| {
                        move_anim
                            .anim
                            .animate(&el, prev_snapshot, new_snapshot, animate_size)
                    }));
                }
            });
        });
    });

    let items_fn = move || {
        alive_items.with(|items| {
            leaving_items.with(|leaving_items| {
                items
                    .keys()
                    .chain(leaving_items.keys())
                    .cloned()
                    .collect::<Vec<_>>()
            })
        })
    };

    let children_fn = {
        {
            let wrapped_children = Rc::new(as_child_of_current_owner(move |k: K| {
                alive_items.with_untracked(|alive_items| {
                    leaving_items.with_untracked(|leaving_items| {
                        alive_items
                            .get(&k)
                            .or_else(|| leaving_items.get(&k))
                            .map(|item| children(item))
                    })
                })
            }));

            move |k: K| {
                let (view, scope) = wrapped_children(k.clone());

                let Some(view) = view else {
                    return ().into_view();
                };

                let view = view.into_view();

                let el = if is_server() {
                    None
                } else {
                    Some(extract_el_from_view(&view).expect("Could not extract element from view"))
                };

                alive_items_meta.update_value(|meta| {
                    meta.insert(
                        k,
                        ItemMeta {
                            el,
                            scope,
                            cur_anim: None,
                        },
                    );
                });

                view
            }
        }
    };

    view! {
        <For each=items_fn.clone() key=move |k| k.clone() children=children_fn.clone() />
    }
}

pub fn extract_el_from_view(view: &View) -> anyhow::Result<web_sys::HtmlElement> {
    use wasm_bindgen::JsCast;
    match view {
        View::Component(component) => {
            let node_view = component
                .children
                .first()
                .ok_or_else(|| anyhow::anyhow!("No children in component"))?;
            extract_el_from_view(node_view)
        }
        View::Element(view) => {
            let el = view
                .clone()
                .into_html_element()
                .dyn_ref::<web_sys::HtmlElement>()
                .ok_or_else(|| {
                    anyhow::anyhow!("Could not convert leptos::HtmlElement to web_sys::HtmlElement")
                })?
                .clone();

            Ok(el)
        }
        v => Err(anyhow::anyhow!(
            "Could not extract element from view: {:?}",
            v
        )),
    }
}

#[component]
pub fn LxAnimatedShow(
    children: ChildrenFn,
    when: Signal<bool>,
    #[prop(default = false)] appear: bool,
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
        <AnimatedFor each key=|_| 0 children=children_fn appear />
    }
}

#[component]
pub fn AnimatedSwap(
    content: Signal<View>,
    #[prop(default = false)] appear: bool,
    #[prop(default = false)] handle_margins: bool,
    #[prop(default = FadeAnimation::default().into(), into)] enter_anim: AnyEnterAnimation,
    #[prop(default = FadeAnimation::default().into(), into)] leave_anim: AnyLeaveAnimation,
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

fn get_el_snapshot(
    el: &web_sys::HtmlElement,
    record_extent: bool,
    handle_margins: bool,
) -> ElementSnapshot {
    let extent = record_extent
        .then(|| {
            let rect = el.get_bounding_client_rect();
            Extent {
                width: rect.width(),
                height: rect.height(),
            }
        })
        .unwrap_or_default();

    if handle_margins {
        el.style().set_property("margin", "0px").unwrap();
    }

    let position = Position {
        x: el.offset_left() as f64,
        y: el.offset_top() as f64,
    };

    if handle_margins {
        el.style().remove_property("margin").unwrap();
    }

    ElementSnapshot { position, extent }
}

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
