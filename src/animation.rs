use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use indexmap::IndexMap;
use leptos::html::AnyElement;
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
}

#[derive(serde::Serialize)]
struct EnterLeaveKeyframe {
    opacity: f64,
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
    easing: Option<&'static str>,
) -> Animation {
    #[cfg(not(feature = "ssr"))]
    {
        use web_sys::KeyframeAnimationOptions;
        let mut options = KeyframeAnimationOptions::new();

        options.duration(duration).fill(fill_mode);

        if let Some(easing) = easing {
            options.easing(easing);
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
struct ElementSnapshot {
    position: Position,
    extent: Extent,
}

pub const ENTER_LEAVE_DURATION: f64 = 500.0;
pub const MOVE_DURATION: f64 = 500.0;

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
                            )
                        }
                    })
                })
                .collect::<HashMap<_, _>>()
        });

        // Move leaving items back to alive if they are re-added during the animation
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
                            let Some(ItemMeta { el, scope }) = alive_items_meta.remove(k) else {
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

                            let values = vec![1.0, 0.0];

                            let width = el.offset_width();
                            let height = el.offset_height();

                            let style = el.style();
                            style.set_property("position", "absolute").unwrap();
                            style
                                .set_property("top", &format!("{}px", snapshot.position.y))
                                .unwrap();
                            style
                                .set_property("left", &format!("{}px", snapshot.position.x))
                                .unwrap();

                            style
                                .set_property("width", &format!("{}px", width))
                                .unwrap();

                            style
                                .set_property("height", &format!("{}px", height))
                                .unwrap();

                            let arr: Array = values
                                .into_iter()
                                .map(|opacity| {
                                    serde_wasm_bindgen::to_value(&EnterLeaveKeyframe { opacity })
                                        .unwrap()
                                })
                                .collect();

                            let anim = animate(
                                &el,
                                Some(&arr.into()),
                                &ENTER_LEAVE_DURATION.into(),
                                FillMode::None,
                                None,
                            );

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
            alive_items_meta.with_value(|items| {
                for (k, meta) in items.iter() {
                    let el = meta.el.clone().expect("el always exists on the client");
                    let Some(&prev_snapshot) = snapshots.get(k) else {
                        // Enter-animation

                        if let Some(on_enter_start) = on_enter_start {
                            on_enter_start(el.clone());
                        }

                        let values = vec![0.0, 1.0];

                        let arr: Array = values
                            .into_iter()
                            .map(|opacity| {
                                serde_wasm_bindgen::to_value(&EnterLeaveKeyframe { opacity })
                                    .unwrap()
                            })
                            .collect();

                        animate(
                            &el,
                            Some(&arr.into()),
                            &ENTER_LEAVE_DURATION.into(),
                            FillMode::None,
                            None,
                        );

                        continue;
                    };

                    // Move-animation

                    let new_snapshot = get_el_snapshot(&el, animate_size);

                    if prev_snapshot == new_snapshot {
                        continue;
                    }

                    let diff = prev_snapshot.position - new_snapshot.position;

                    let arr: Array = [
                        serde_wasm_bindgen::to_value(&MoveAnimKeyframe {
                            transform_origin: "top left".to_string(),
                            transform: format!("translate({}px, {}px)", diff.x, diff.y),
                            width: animate_size
                                .then(|| format!("{}px", prev_snapshot.extent.width)),
                            height: animate_size
                                .then(|| format!("{}px", prev_snapshot.extent.height)),
                        })
                        .unwrap(),
                        serde_wasm_bindgen::to_value(&MoveAnimKeyframe {
                            transform_origin: "top left".to_string(),
                            transform: "none".to_string(),
                            width: animate_size.then(|| format!("{}px", new_snapshot.extent.width)),
                            height: animate_size
                                .then(|| format!("{}px", new_snapshot.extent.height)),
                        })
                        .unwrap(),
                    ]
                    .into_iter()
                    .collect();

                    animate(
                        &el,
                        Some(&arr.into()),
                        &MOVE_DURATION.into(),
                        FillMode::None,
                        Some("ease-in-out"),
                    );
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
                    meta.insert(k, ItemMeta { el, scope });
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
pub fn AnimatedSwap(content: Signal<View>, #[prop(default = false)] appear: bool) -> impl IntoView {
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
        <AnimatedFor each key=move |k| *k children=children_fn appear />
    }
}

/// Delays a signal to the end of a tick.
pub fn delay_signal<T: Clone>(source_signal: impl IntoSignal<Value = T>) -> ReadSignal<T> {
    let source_signal = source_signal.into_signal();
    let s = RwSignal::new(source_signal.get_untracked());

    create_isomorphic_effect(move |_| {
        let v = source_signal.get();
        queue_microtask(move || s.set(v));
    });

    s.read_only()
}

fn get_el_snapshot(el: &web_sys::HtmlElement, record_extent: bool) -> ElementSnapshot {
    let extent = record_extent
        .then(|| {
            let rect = el.get_bounding_client_rect();
            Extent {
                width: rect.width(),
                height: rect.height(),
            }
        })
        .unwrap_or_default();

    el.style().set_property("margin", "0px").unwrap();
    let position = Position {
        x: el.offset_left() as f64,
        y: el.offset_top() as f64,
    };
    el.style().remove_property("margin").unwrap();

    ElementSnapshot { position, extent }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ResizeAndMoveAnimKeyframe {
    transform_origin: String,
    transform: String,
    width: String,
    height: String,
    position: &'static str,
    top: String,
    left: String,
}

/// Animates position and size of an element.
pub fn animated_size_and_pos(el: HtmlElement<AnyElement>, about_to_change: Trigger) {
    create_effect(move |prev| {
        about_to_change.track();
        if prev.is_none() {
            return;
        }

        // First get the current values!
        let initial = el.get_bounding_client_rect();

        // Need to wait a bit for the change that triggered about_to_change to complete
        queue_microtask({
            let el = el.clone();
            move || {
                queue_microtask(move || {
                    let snapshot = get_el_snapshot(&el, false);
                    let new_pos = snapshot.position;
                    let new_val = el.get_bounding_client_rect();

                    let duration = 100.0;

                    let arr: Array = [
                        serde_wasm_bindgen::to_value(&ResizeAndMoveAnimKeyframe {
                            transform_origin: "top left".to_string(),
                            transform: format!(
                                "translate({}px, {}px)",
                                initial.x() - new_val.x(),
                                initial.y() - new_val.y(),
                            ),
                            width: format!("{}px", initial.width()),
                            height: format!("{}px", initial.height()),
                            position: "absolute",
                            top: format!("{}px", new_pos.y),
                            left: format!("{}px", new_pos.x),
                        })
                        .unwrap(),
                        serde_wasm_bindgen::to_value(&ResizeAndMoveAnimKeyframe {
                            transform_origin: "top left".to_string(),
                            transform: "none".to_string(),
                            width: format!("{}px", new_val.width()),
                            height: format!("{}px", new_val.height()),
                            position: "absolute",
                            top: format!("{}px", new_pos.y),
                            left: format!("{}px", new_pos.x),
                        })
                        .unwrap(),
                    ]
                    .into_iter()
                    .collect();

                    animate(
                        &el,
                        Some(&arr.into()),
                        &duration.into(),
                        FillMode::None,
                        Some("ease-out"),
                    );
                });
            }
        });
    });
}

pub struct LayoutEntry<K: Hash + Eq + Clone + 'static> {
    pub key: K,
    pub view_fn: Box<dyn Fn() -> View>,
}

pub struct LayoutResult<K: Hash + Eq + Clone + 'static> {
    pub class: Option<String>,
    pub entries: Vec<LayoutEntry<K>>,
}

#[component]
pub fn AnimatedLayout<K, ContentsFn>(contents: ContentsFn) -> impl IntoView
where
    K: Hash + Eq + Clone + 'static,
    ContentsFn: Fn() -> LayoutResult<K> + 'static,
{
    let new_class = StoredValue::new(None::<String>);
    let class = RwSignal::new(None::<String>);

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

    view! {
        <div class=class>
            <AnimatedFor each key children on_after_snapshot />
        </div>
    }
}
