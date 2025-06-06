use crate::{EnterAnimation, FadeAnimation, LeaveAnimation, MoveAnimation, SlidingAnimation};
use indexmap::IndexMap;
use leptos::either::Either;
use leptos::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use wasm_bindgen::closure::Closure;
use web_sys::js_sys::Array;
use web_sys::{js_sys, HtmlElement};
use web_sys::{Animation, FillMode};

use crate::position::{Extent, Position};

/// Metadata for each item that's currently alive in the AnimatedFor.
struct ItemMeta {
    /// Reference to the HTML element, if we found one
    el: Option<web_sys::HtmlElement>,

    /// Reference to the scope which will be dropped when the item is removed.
    /// Used to prevent reactive state changes during the leave-animation.
    observer: Option<Owner>,

    /// The current animation that's running on the element.
    /// We want to cancel this animation when we start a new one so that we don't have two running
    /// at the same time.
    cur_anim: Option<Animation>,
}

/// Keyframe for the FLIP animation.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct MoveAnimKeyframe {
    transform_origin: String,
    transform: String,

    /// Only set if `animate_size` is true
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<String>,

    /// Only set if `animate_size` is true
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<String>,
}

/// Wrapper around the `animate` function in the Web Animations API because in web_sys it is still
/// unstable and that causes some problems with cranelift.
pub fn animate(
    el: &HtmlElement,
    keyframes: Option<&js_sys::Object>,
    duration: &::wasm_bindgen::JsValue,
    fill_mode: FillMode,
    easing: Option<impl AsRef<str>>,
) -> Animation {
    #[cfg(not(feature = "ssr"))]
    {
        use web_sys::KeyframeAnimationOptions;
        let options = KeyframeAnimationOptions::new();

        options.set_duration(duration);
        options.set_fill(fill_mode);

        if let Some(easing) = easing {
            options.set_easing(easing.as_ref());
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

/// A snapshot of an element's position and size at a specific moment.
#[derive(Clone, Copy, Debug, Default)]
pub struct ElementSnapshot {
    /// The position of the element.
    position: Position,

    /// The height and width of the element.
    extent: Option<Extent>,
}

/// Wrapper trait for [`EnterAnimation`] to be used as a dyn trait. The original trait is not
/// object-safe because it has an associated type.
trait EnterAnimationHandler {
    /// Run the enter-animation. The returned `Animation` may be used to cancel the animation later
    /// as well as to trigger a callback when the animation finishes.
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation;
}

/// Automatically implemented on all `EnterAnimation`s.
impl<T: EnterAnimation> EnterAnimationHandler for T {
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation {
        let r = self.enter();

        // Build the JavaScript object from the animations keyframes.
        let arr: Array = r
            .keyframes
            .into_iter()
            .map(|v| serde_wasm_bindgen::to_value(&v).unwrap())
            .collect();

        animate(
            &el,
            Some(&arr.into()),
            &(r.duration.as_secs_f64() * 1000.0).into(),
            // The fill mode can shadow timing bugs, so we avoid it as much as possible.
            FillMode::None,
            r.timing_fn.as_ref().map(|v| v.as_str()),
        )
    }
}

/// Any struct that implements [`EnterAnimation`] can be converted into this using `into()`.
/// The props on the various components will do this automatically.
pub struct AnyEnterAnimation {
    anim: Box<dyn EnterAnimationHandler>,
}

/// Any [`EnterAnimation`] can be converted to an [`AnyEnterAnimation`] using the intermediate
/// dyn Trait.
impl<T: EnterAnimationHandler + 'static> From<T> for AnyEnterAnimation {
    fn from(v: T) -> Self {
        AnyEnterAnimation { anim: Box::new(v) }
    }
}

/// Wrapper trait for [`LeaveAnimation`] to be used as a dyn trait. The original trait is not
/// object-safe because it has an associated type.
trait LeaveAnimationHandler {
    fn animate(&self, el: &HtmlElement) -> Animation;
}

/// Automatically implemented on all `LeaveAnimation`s.
impl<T: LeaveAnimation> LeaveAnimationHandler for T {
    fn animate(&self, el: &HtmlElement) -> Animation {
        let r = self.leave(el);

        // Build the JavaScript object from the animations keyframes.
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

/// Any struct that implements [`LeaveAnimation`] can be converted into this using `into()`.
/// The props on the various components will do this automatically.
#[derive(Clone)]
pub struct AnyLeaveAnimation {
    anim: Rc<dyn LeaveAnimationHandler>,
}

/// Any [`LeaveAnimation`] can be converted to an [`AnyLeaveAnimation`] using the intermediate dyn Trait.
impl<T: LeaveAnimationHandler + 'static> From<T> for AnyLeaveAnimation {
    fn from(v: T) -> Self {
        AnyLeaveAnimation { anim: Rc::new(v) }
    }
}

/// Wrapper trait for [`MoveAnimation`] to be used as a dyn trait. The original trait is not
/// object-safe because it has an associated type.
trait MoveAnimationHandler {
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

        // Build the JavaScript object. Move Animations don't support keyframes yet.
        let arr: Array = [
            serde_wasm_bindgen::to_value(&MoveAnimKeyframe {
                transform_origin: "top left".to_string(),
                transform: format!("translate({}px, {}px)", diff.x, diff.y),
                width: animate_size.then(|| format!("{}px", prev_snapshot.extent.unwrap().width)),
                height: animate_size.then(|| format!("{}px", prev_snapshot.extent.unwrap().height)),
            })
            .unwrap(),
            serde_wasm_bindgen::to_value(&MoveAnimKeyframe {
                transform_origin: "top left".to_string(),
                transform: "none".to_string(),
                width: animate_size.then(|| format!("{}px", new_snapshot.extent.unwrap().width)),
                height: animate_size.then(|| format!("{}px", new_snapshot.extent.unwrap().height)),
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

/// Any struct that implements [`MoveAnimation`] can be converted into this using `into()`.
pub struct AnyMoveAnimation {
    anim: Box<dyn MoveAnimationHandler>,
}

/// Any [`MoveAnimation`] can be converted to an [`AnyMoveAnimation`] using the intermediate
/// dyn Trait.
impl<T: MoveAnimationHandler + 'static> From<T> for AnyMoveAnimation {
    fn from(v: T) -> Self {
        AnyMoveAnimation { anim: Box::new(v) }
    }
}

/// A version of the [`<For />`][leptos::For] component that animates children when they enter or
/// leave, as well as moving them around when their position changes.
///
/// # Example
/// ```
/// use leptos::prelude::*;
/// use leptos_animate::{AnimatedFor, FadeAnimation, DynamicsAnimation};
/// use std::time::Duration;
///
/// #[component]
/// pub fn MyGrid() -> impl IntoView {
///     let next_key = StoredValue::new(6);
///     let elements = RwSignal::new(vec![1, 2, 3, 4, 5]);
///
///     let get_next_key = move || {
///         let v = next_key.get_value();
///         next_key.update_value(|v| *v += 1);
///         v
///     };
///
///     let insert_first = move |_| {
///         elements.update(|v| {
///             v.insert(0, get_next_key());
///         })
///     };
///
///     let remove_first = move |_| {
///         elements.update(|v| {
///             v.remove(0);
///         })
///     };
///
///     let each = move || elements.get();
///
///     let children = move |c: &i32| {
///         let c = *c;
///         view! {
///             <div class="element">{c}</div>
///         }
///     };
///
///     // Unique key for each item
///     let key = move |v: &i32| *v;
///     
///     // Optional enter animations
///     let enter_anim = FadeAnimation::new(Duration::from_millis(500), "ease-out");
///     let leave_anim = FadeAnimation::new(Duration::from_millis(500), "ease-out");
///     let move_anim = DynamicsAnimation::new(2.0, 0.65, 0.0);
///
///     view! {
///         <button on:click=insert_first>"+ Add"</button>
///         <button on:click=remove_first>"- Remove"</button>
///         <div style="display:grid;grid-template-columns: 100px 100px 200px;">
///             <AnimatedFor each key children animate_size=true enter_anim leave_anim move_anim />
///         </div>
///     }
/// }
/// ```
#[component]
pub fn AnimatedFor<IF, I, T, EF, N, KF, K>(
    /// A signal-like function that returns the items to iterate over.
    ///
    /// Please note, unlike on [`<For />`][leptos::For], the items are stored inside this component
    /// and only references to them are passed to the `children`. This is because `AnimatedFor`
    /// actually renders the items in an underlying `For` component whose `each` function has to be
    /// rerun more frequently than this one.
    each: IF,
    /// A function that returns a key that is unique for each item currently in the list.
    key: KF,
    /// A function that receives a reference to the item and returns the view to render it.
    /// Just like on the [`<For />`][leptos::For] component, this will only rerun if the item with
    /// the key is being removed and then re-added later.
    ///
    /// **Please note**, unlike the [`<For />`][leptos::For] component, this only gets a reference,
    /// not the original value. If you need to take ownership of the item, you need to clone or
    /// copy it.
    ///
    /// The returned View must have a DOM node as its top level element, or a component that does.
    /// Due to the way leptos works, we cannot currently extract node-refs from other elements such
    /// as `Suspense`, `DynChild`, `Each`, etc. Also Fragments/Components that return multiple
    /// elements will only have their first element animated.
    ///
    /// The elements should be able to handle being set to `position:absolute` during the
    /// leave-animation, although it will fix their size in place (so for example an element with
    /// `width:100%` will still work). Ideally the elements should also be block-like elements
    /// without margins.
    children: EF,
    /// Callback that is called for each item when it is about to start its leaving animation
    /// after it has been snapshotted. Useful to handle additional style changes that happen at the
    /// same time when `each` changes, for example if you want to apply a counter-animation. Note
    /// that leaving items are set to `position:absolute`.
    ///
    /// See also [`AnimatedLayout`][crate::AnimatedLayout].
    #[prop(optional)]
    on_leave_start: Option<Callback<(web_sys::HtmlElement, Position)>>,
    /// See `on_leave_start`.
    #[prop(optional)]
    on_enter_start: Option<Callback<web_sys::HtmlElement>>,
    /// Callback that is called after the initial snapshots of all elements have been taken but
    /// before the goal snapshots are taken. This is the time to apply CSS changes to the elements
    /// or to the container and have the elements be able to animate to their new positions.
    #[prop(optional)]
    on_after_snapshot: Option<Callback<()>>,
    /// Whether enter animations play when the component is initially rendered. This is usually not
    /// what you want. On SSR this will cause visual glitches because the enter animation would
    /// start much later than the initial render.
    #[prop(default = false)]
    appear: bool,
    /// Whether to also animate the sizes of the elements for move animations, for example in a
    /// grid with differently sized columns or rows.
    ///
    /// Please note this only works for sizes that are specified "top-down",
    /// like column widths with `px`, `%` or `fr` as their units. It will not work for sizes that
    /// depend on the contents such as `grid-template-columns:auto 1fr`. This is because those
    /// columns will see the size during the entire move animation and therefore would adjust
    /// their own size during the animation. [`SizeTransition`][crate::SizeTransition] can handle
    /// that case in some situations.
    #[prop(default = false)]
    animate_size: bool,
    /// The enter animation to use for new elements.
    #[prop(default = FadeAnimation::default().into(), into)]
    enter_anim: AnyEnterAnimation,
    /// The leave animation to use for elements that are removed.
    #[prop(default = FadeAnimation::default().into(), into)]
    leave_anim: AnyLeaveAnimation,
    /// The move animation to use for elements that change position.
    #[prop(default = SlidingAnimation::default().into(), into)]
    move_anim: AnyMoveAnimation,
    /// Whether to use the window's scroll position for the snapshots. This is useful if the
    /// window gets scrolled during the transition, most likely due to it being a route transition.
    #[prop(default = false)]
    compensate_window_scroll: bool,

    #[prop(optional)] enter_anim_override: Option<Box<dyn Fn(&T) -> Option<AnyEnterAnimation>>>,

    #[prop(optional)] leave_anim_override: Option<Box<dyn Fn(&T) -> Option<AnyLeaveAnimation>>>,
) -> impl IntoView
where
    IF: Fn() -> I + Send + Sync + 'static,
    I: IntoIterator<Item = T> + 'static,
    EF: Fn(&T) -> N + Send + Sync + 'static,
    N: IntoView + 'static,
    KF: Fn(&T) -> K + Send + Sync + Clone + 'static,
    K: Eq + Hash + Clone + Send + Sync + 'static,
    T: 'static,
{
    let key_fn = StoredValue::new(key);

    let alive_items = RwSignal::new_local(IndexMap::<K, T>::new());
    let leaving_items = RwSignal::new_local(IndexMap::<K, T>::new());

    let alive_items_meta = StoredValue::new_local(HashMap::<K, ItemMeta>::new());

    let enter_anim = StoredValue::new_local(enter_anim);
    let leave_anim = StoredValue::new_local(leave_anim);
    let move_anim = StoredValue::new_local(move_anim);

    let enter_anim_override = enter_anim_override.map(|v| StoredValue::new_local(v));
    let leave_anim_override = leave_anim_override.map(|v| StoredValue::new_local(v));

    // Listen to changes in `each`. This handles all the animations.
    let e = RenderEffect::new_isomorphic(move |prev: Option<()>| {
        let new_items = each()
            .into_iter()
            .map(|i| (key_fn.with_value(|k| k(&i)), i))
            .collect::<IndexMap<_, _>>();

        // Need to skip all the animations on SSR
        let is_hydrating = Owner::current_shared_context().unwrap().during_hydration();
        if cfg!(feature = "ssr") || is_hydrating {
            alive_items_meta.update_value(|meta| {
                meta.clear();
            });
            alive_items.update(move |items| {
                *items = new_items;
            });
            if let Some(on_after_snapshot) = on_after_snapshot {
                on_after_snapshot.run(());
            }
            return;
        }

        // Get initial snapshots of all previously alive elements
        let snapshots = alive_items_meta.with_value(|alive_items_meta| {
            alive_items_meta
                .iter()
                .filter_map(|(k, meta)| {
                    let el = meta.el.as_ref().expect("el always exists on the client");
                    if !el.is_connected() {
                        return None;
                    }

                    // Needs to also record the extent for the leave animations to work properly.
                    Some((k.clone(), get_el_snapshot(el, true)))
                })
                .collect::<HashMap<_, _>>()
        });

        // Items that are re-added during the animation while they are still leaving must be
        // removed from the leaving_items list and will then be treated as new elements (Their
        // scope already got disposed, so there's no way to resurrect them).
        // TODO: Now with pause() and resume() there might actually be a way to resurrect them.
        for k in new_items.keys() {
            if leaving_items.with_untracked(|leaving_items| leaving_items.contains_key(k)) {
                leaving_items.update(|leaving_items| {
                    leaving_items.swap_remove(k);
                });
            }
        }

        // Callback trigger for CSS changes to be applied after snapshots
        if let Some(on_after_snapshot) = on_after_snapshot {
            on_after_snapshot.run(());
        }

        // Update alive items and trigger leave-animations
        alive_items.update({
            let snapshots = &snapshots;
            move |alive_items| {
                let items_to_remove = alive_items
                    .drain(..)
                    .filter(|(k, _)| !new_items.contains_key(k))
                    .collect::<Vec<_>>();

                alive_items_meta.update_value(|alive_items_meta| {
                    for (k, item) in items_to_remove.iter() {
                        let Some(ItemMeta {
                            el,
                            observer,
                            cur_anim,
                        }) = alive_items_meta.remove(k)
                        else {
                            continue;
                        };

                        if let Some(o) = observer.clone() {
                            o.pause();
                        }

                        let el = el.expect("el always exists on the client");

                        let Some(snapshot) = snapshots.get(k) else {
                            continue;
                        };

                        if let Some(on_leave_start) = on_leave_start {
                            on_leave_start.run((el.clone(), snapshot.position));
                        }

                        let extent = snapshot.extent;

                        let style = el.style();
                        style.set_property("position", "absolute").unwrap();
                        style
                            .set_property("top", &format!("{}px", snapshot.position.y))
                            .unwrap();
                        style
                            .set_property("left", &format!("{}px", snapshot.position.x))
                            .unwrap();

                        style
                            .set_property("width", &format!("{}px", extent.unwrap().width))
                            .unwrap();

                        style
                            .set_property("height", &format!("{}px", extent.unwrap().height))
                            .unwrap();

                        let k = k.clone();

                        let override_anim = leave_anim_override.and_then(|anim_override| {
                            anim_override.try_with_value(|e| e(item)).flatten()
                        });

                        let anim_fn = move || {
                            let leave_anim_normal = leave_anim.read_value();
                            // Get overridden animation if it exists
                            let actual_leave_anim = override_anim
                                .as_ref()
                                .unwrap_or_else(|| &*leave_anim_normal);

                            // animate()
                            let anim = actual_leave_anim.anim.animate(&el);

                            // Cancel the animation after the leave animation had a chance to
                            // read out its properties.
                            if let Some(cur_anim) = cur_anim.as_ref() {
                                cur_anim.cancel();
                            }

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
                        };

                        if compensate_window_scroll {
                            let snap_y = snapshot.position.y;
                            let old_window_pos = window().scroll_y().unwrap();

                            let has_animated = Arc::new(Mutex::new(false));

                            let cb = Closure::<dyn Fn()>::new({
                                let has_animated = Arc::clone(&has_animated);
                                let anim_fn = anim_fn.clone();
                                move || {
                                    let new_window_pos = window().scroll_y().unwrap();

                                    style
                                        .set_property(
                                            "top",
                                            &format!(
                                                "{}px",
                                                snap_y + new_window_pos - old_window_pos
                                            ),
                                        )
                                        .unwrap();

                                    let _ = window().scroll_y().unwrap();

                                    anim_fn();
                                    *has_animated.lock().unwrap() = true;
                                }
                            });

                            let listener = cb.into_js_value().into();

                            window()
                                .add_event_listener_with_callback("scroll", &listener)
                                .unwrap();

                            let remove_handler = Closure::<dyn Fn()>::new(move || {
                                window()
                                    .remove_event_listener_with_callback("scroll", &listener)
                                    .unwrap();

                                if !*has_animated.lock().unwrap() {
                                    anim_fn();
                                }
                            });

                            window()
                                .set_timeout_with_callback(&remove_handler.into_js_value().into())
                                .unwrap();
                        } else {
                            anim_fn();
                        }
                    }
                });

                leaving_items.update(move |leaving_items| {
                    leaving_items.extend(items_to_remove);
                });
                alive_items.extend(new_items);
            }
        });

        if prev.is_none() && !appear {
            return;
        }

        // Wait for the children to be created so that we get element refs for enter-animation
        queue_microtask(move || {
            alive_items_meta.try_update_value(|items| {
                for (k, meta) in items.iter_mut() {
                    let el = meta.el.clone().expect("el always exists on the client");
                    let Some(&prev_snapshot) = snapshots.get(k) else {
                        // Enter-animation

                        if let Some(on_enter_start) = on_enter_start {
                            on_enter_start.run(el.clone());
                        }

                        // Cancel previous animation
                        meta.cur_anim.take().map(|cur_anim| cur_anim.cancel());

                        // Get overridden animation if it exists
                        let override_anim = enter_anim_override.and_then(|enter_anim_override| {
                            let items = alive_items.read_untracked();
                            items.get(k).and_then(|i| {
                                enter_anim_override.try_with_value(|e| e(i)).flatten()
                            })
                        });

                        // animate()
                        meta.cur_anim = Some(if let Some(override_anim) = override_anim {
                            override_anim.anim.animate(&el)
                        } else {
                            enter_anim.with_value(|enter_anim| enter_anim.anim.animate(&el))
                        });

                        continue;
                    };

                    // Move-animation
                    let new_snapshot = get_el_snapshot(&el, animate_size);

                    // if animate_size is set to false then the old extent is still Some() but the
                    // new extent is None, and we don't want that to trigger a move.
                    if prev_snapshot.position == new_snapshot.position
                        && (new_snapshot.extent.is_none()
                            || new_snapshot.extent == prev_snapshot.extent)
                    {
                        continue;
                    }

                    meta.cur_anim.take().map(|cur_anim| cur_anim.cancel());

                    meta.cur_anim = Some(move_anim.with_value(|move_anim| {
                        move_anim
                            .anim
                            .animate(&el, prev_snapshot, new_snapshot, animate_size)
                    }));
                }
            });
        });
    });

    on_cleanup(move || drop(e));

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

    let children = Arc::new(children);

    let children_fn = {
        // Register children refs and scopes.
        move |k: K| {
            let children = children.clone();
            let k = Arc::new(k);
            move || {
                let k = Arc::clone(&k);

                let observer = Owner::current();

                let view = alive_items.with_untracked(|alive_items| {
                    leaving_items.with_untracked(|leaving_items| {
                        alive_items
                            .get(k.as_ref())
                            .or_else(|| leaving_items.get(k.as_ref()))
                            .map(|item| children(item))
                    })
                });

                let Some(view) = view else {
                    return Either::Left(().into_view());
                };

                let add_to_meta = move |el: Option<HtmlElement>| {
                    alive_items_meta.update_value(|meta| {
                        meta.insert(
                            K::clone(k.as_ref()),
                            ItemMeta {
                                el,
                                observer: observer.clone(),
                                cur_anim: None,
                            },
                        );
                    });
                };

                #[cfg(feature = "ssr")]
                add_to_meta(None);

                #[cfg(not(feature = "ssr"))]
                let view = view.directive(
                    move |el: web_sys::Element| {
                        use wasm_bindgen::JsCast;
                        add_to_meta(Some(el.dyn_into().unwrap()));
                    },
                    (),
                );

                Either::Right(view)
            }
        }
    };

    view! {
        <For each=items_fn.clone() key=move |k| k.clone() children=children_fn.clone() />
    }
}

/// Take a snapshot of an element's position and (optionally) size.
fn get_el_snapshot(el: &HtmlElement, record_extent: bool) -> ElementSnapshot {
    // Using 2 bounding rects instead of "offset" due to subpixel issues.
    let el_bounding = el.get_bounding_client_rect();
    let p_bounding = el
        .offset_parent()
        .unwrap_or_else(|| leptos::tachys::dom::body().into())
        .get_bounding_client_rect();

    // offset / bounding rects don't include margins.
    let css_props = window().get_computed_style(&el).unwrap().unwrap();
    let margin_top = css_props.get_property_value("margin-top").unwrap();
    let margin_left = css_props.get_property_value("margin-left").unwrap();

    let extent = record_extent.then(|| Extent {
        width: el_bounding.width(),
        height: el_bounding.height(),
    });

    let margin_top = margin_top
        .strip_suffix("px")
        .expect("margin-top is not in pixels")
        .parse::<f64>()
        .unwrap();
    let margin_left = margin_left
        .strip_suffix("px")
        .expect("margin-left is not in pixels")
        .parse::<f64>()
        .unwrap();

    let position = Position {
        x: el_bounding.x() - p_bounding.x() - margin_left,
        y: el_bounding.y() - p_bounding.y() - margin_top,
    };

    ElementSnapshot { position, extent }
}
