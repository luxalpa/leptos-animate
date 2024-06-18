use std::rc::Rc;

use crate::{animate, Extent, ResizeAnimation, SlidingAnimation};
use leptos::html::AnyElement;
use leptos::{component, view, Children, HtmlElement, IntoView, StoredValue};
use leptos_use::use_resize_observer;
use web_sys::js_sys::Array;
use web_sys::{FillMode, ResizeObserverSize};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SizeTransitionKeyframe {
    margin_right: String,
}

/// Note: Only works for elements that infer their width from their contents;
/// Does not work for elements that infer their width from their parents (like 1fr grid items or width:100%).
#[component]
pub fn SizeTransition(children: Children) -> impl IntoView {
    view! {
        <span style="display:inline-block" use:animated_size=SlidingAnimation::default()>
            {children()}
        </span>
    }
}

trait SizeTransitionHandler {
    fn animate(&self, el: HtmlElement<AnyElement>, current_width: f64, goal_width: f64);
}

impl<T: ResizeAnimation> SizeTransitionHandler for T {
    fn animate(&self, el: HtmlElement<AnyElement>, current_width: f64, goal_width: f64) {
        let r = self.animate(
            Extent {
                width: current_width,
                height: 0.0,
            },
            Extent {
                width: goal_width,
                height: 0.0,
            },
        );

        let arr: Array = [current_width, goal_width]
            .into_iter()
            .map(|width| {
                serde_wasm_bindgen::to_value(&SizeTransitionKeyframe {
                    margin_right: format!("{}px", width - goal_width),
                })
                .unwrap()
            })
            .collect();

        animate(
            &el,
            Some(&arr.into()),
            &(r.duration.as_secs_f64() * 1000.0).into(),
            FillMode::None,
            r.timing_fn.as_ref().map(|v| v.as_str()),
        );
    }
}

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

pub fn animated_size(el: HtmlElement<AnyElement>, size_anim: AnySizeTransitionAnimation) {
    let last_width = StoredValue::new(None::<f64>);

    use_resize_observer((&*el).clone(), move |entries, _| {
        let rects = entries[0].border_box_size();
        let rect: ResizeObserverSize = rects.get(0).into();
        let goal_width = rect.inline_size();

        let Some(current_width) = last_width.get_value() else {
            last_width.set_value(Some(goal_width));
            return;
        };

        last_width.set_value(Some(goal_width));

        // Animate!

        size_anim
            .anim
            .animate(el.clone(), current_width, goal_width);
    });
}
