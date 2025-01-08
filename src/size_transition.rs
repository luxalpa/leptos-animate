use std::rc::Rc;

use crate::{animate, Extent, ResizeAnimation, SlidingAnimation};
use leptos::prelude::*;
use leptos::tachys::renderer::dom::Element;
use leptos_use::use_resize_observer;
use wasm_bindgen::JsCast;
use web_sys::js_sys::Array;
use web_sys::{FillMode, ResizeObserverSize};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SizeTransitionKeyframe {
    margin_right: String,
    margin_bottom: String,
}

/// Animates the size of its contents whenever that changes.
///
/// Note: Only works for elements that infer their size from their contents;
/// Does not work for elements that infer their size from their parents (like 1fr grid items or width:100%).
///
/// Uses a ResizeObserver to listen for size changes. Wraps the children in a span with `display:inline-block` and `position:relative`.
///
/// **Note:** The size is animated using `margin-right` (for width) and margin-bottom (for height) instead of `width`/`height` in order to not trip up the underlying `ResizeObserver`.
#[component]
pub fn SizeTransition(
    children: Children,
    #[prop(into, default=SlidingAnimation::default().into())]
    resize_anim: AnySizeTransitionAnimation,
) -> impl IntoView {
    view! {
        <span style="display:inline-block; position:relative;" use:animated_size=resize_anim>
            {children()}
        </span>
    }
}

trait SizeTransitionHandler {
    fn animate(&self, el: Element, snapshot: Extent, new_snapshot: Extent);
}

impl<T: ResizeAnimation> SizeTransitionHandler for T {
    fn animate(&self, el: Element, snapshot: Extent, new_snapshot: Extent) {
        let r = self.animate(snapshot, new_snapshot);

        let arr: Array = [snapshot, new_snapshot]
            .into_iter()
            .map(|snapshot| {
                serde_wasm_bindgen::to_value(&SizeTransitionKeyframe {
                    margin_right: format!("{}px", snapshot.width - new_snapshot.width),
                    margin_bottom: format!("{}px", snapshot.height - new_snapshot.height),
                })
                .unwrap()
            })
            .collect();

        animate(
            el.dyn_ref().unwrap(),
            Some(&arr.into()),
            &(r.duration.as_secs_f64() * 1000.0).into(),
            FillMode::None,
            r.timing_fn.as_ref().map(|v| v.as_str()),
        );
    }
}

/// Any struct that implements [`ResizeAnimation`] can be converted into this using into(). This
/// conversion is typically done automatically.
#[derive(Clone)]
pub struct AnySizeTransitionAnimation {
    anim: Rc<dyn SizeTransitionHandler>,
}

impl<T: SizeTransitionHandler + 'static> From<T> for AnySizeTransitionAnimation {
    fn from(anim: T) -> Self {
        Self {
            anim: Rc::new(anim),
        }
    }
}

impl From<()> for AnySizeTransitionAnimation {
    fn from(_: ()) -> Self {
        SlidingAnimation::default().into()
    }
}

/// Directive to animate the size of an element. See [`SizeTransition`].
///
/// # Usage
/// ```
/// // This is optional, it will default to SlidingAnimation::default() if not provided.
/// let resize_anim = SlidingAnimation::default();
/// view! {
///     <span style="display:inline-block; position:relative;" use:animated_size=resize_anim>
///         <SomeElementThatChangesItsSize />
///     </span>
/// }
/// ```
pub fn animated_size(el: Element, size_anim: AnySizeTransitionAnimation) {
    let snapshot = StoredValue::new(None::<Extent>);

    use_resize_observer(el.clone(), move |entries, _| {
        let rects = entries[0].border_box_size();
        let rect: ResizeObserverSize = rects.get(0).into();
        let new_snapshot = Extent {
            width: rect.inline_size(),
            height: rect.block_size(),
        };

        if let Some(snapshot) = snapshot.get_value() {
            size_anim.anim.animate(el.clone(), snapshot, new_snapshot);
        }

        snapshot.set_value(Some(new_snapshot));
    });
}
