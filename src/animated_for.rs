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

/// Metadata for each item that's currently alive in the AnimatedFor.
struct ItemMeta {
    /// Reference to the HTML element, if we found one
    el: Option<web_sys::HtmlElement>,

    /// Reference to the scope which will be dropped when the item is removed.
    /// Used to prevent reactive state changes during the leave-animation.
    scope: Disposer,

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

/// A snapshot of an element's position and size at a specific moment.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ElementSnapshot {
    /// The position of the element.
    position: Position,

    /// The height and width of the element.
    extent: Extent,
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
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation;
}

/// Automatically implemented on all `LeaveAnimation`s.
impl<T: LeaveAnimation> LeaveAnimationHandler for T {
    fn animate(&self, el: &web_sys::HtmlElement) -> Animation {
        let r = self.leave();

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
pub struct AnyLeaveAnimation {
    anim: Box<dyn LeaveAnimationHandler>,
}

/// Any [`LeaveAnimation`] can be converted to an [`AnyLeaveAnimation`] using the intermediate dyn Trait.
impl<T: LeaveAnimationHandler + 'static> From<T> for AnyLeaveAnimation {
    fn from(v: T) -> Self {
        AnyLeaveAnimation { anim: Box::new(v) }
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

    /// Whether the child elements can have margins applied. This will simply remove the margins
    /// during the snapshotting process for element positions and then reapply them, as such it is
    /// fairly expensive to do. Typically it's better to just wrap your element that has a margin
    /// applied in another element that doesn't. Also this won't handle margins on inline elements
    /// in child-elements (those act really weird!).
    #[prop(default = false)]
    handle_margins: bool,

    /// The enter animation to use for new elements.
    #[prop(default = FadeAnimation::default().into(), into)]
    enter_anim: AnyEnterAnimation,

    /// The leave animation to use for elements that are removed.
    #[prop(default = FadeAnimation::default().into(), into)]
    leave_anim: AnyLeaveAnimation,

    /// The move animation to use for elements that change position.
    #[prop(default = SlidingAnimation::default().into(), into)]
    move_anim: AnyMoveAnimation,
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
    let key_fn = StoredValue::new(key);

    let alive_items = RwSignal::new(IndexMap::<K, T>::new());
    let leaving_items = RwSignal::new(IndexMap::<K, T>::new());

    let alive_items_meta = StoredValue::new(HashMap::<K, ItemMeta>::new());

    let enter_anim = StoredValue::new(enter_anim);
    let leave_anim = StoredValue::new(leave_anim);
    let move_anim = StoredValue::new(move_anim);

    // Listen to changes in `each`. This handles all the animations.
    create_isomorphic_effect(move |prev| {
        let new_items = each()
            .into_iter()
            .map(|i| (key_fn.with_value(|k| k(&i)), i))
            .collect::<IndexMap<_, _>>();

        // Get initial snapshots of all previously alive elements
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

        // Items that are re-added during the animation while they are still leaving must be
        // removed from the leaving_items list and will then be treated as new elements (Their
        // scope already got disposed, so there's no way to resurrect them).
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

        // Update alive items and trigger leave-animations
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

            // Register children refs and scopes.
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

/// Get the node ref from a view. Ideally we'd like to have refs to the comment node or something
/// that this view represents, but that's currently not possible.
fn extract_el_from_view(view: &View) -> anyhow::Result<web_sys::HtmlElement> {
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

/// Take a snapshot of an element's position and (optionally) size.
fn get_el_snapshot(
    el: &web_sys::HtmlElement,
    record_extent: bool,
    handle_margins: bool,
) -> ElementSnapshot {
    let extent = record_extent
        .then(|| {
            // We're using GetBoundingClientRect here because offsetWidth/Height aren't truthful
            // when it comes to paddings.
            let rect = el.get_bounding_client_rect();
            Extent {
                width: rect.width(),
                height: rect.height(),
            }
        })
        .unwrap_or_default();

    // offsetWidth/Height don't include margins.
    if handle_margins {
        el.style().set_property("margin", "0px").unwrap();
    }

    // We're not using GetBoundingClientRect here because the position it returns is in viewport
    // space, but we need it for position:absolute.
    let position = Position {
        x: el.offset_left() as f64,
        y: el.offset_top() as f64,
    };

    if handle_margins {
        el.style().remove_property("margin").unwrap();
    }

    ElementSnapshot { position, extent }
}
