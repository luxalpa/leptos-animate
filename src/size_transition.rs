use crate::animate;
use crate::dynamics::SecondOrderDynamics;
use leptos::{component, html, logging, view, Children, IntoView, NodeRef, StoredValue};
use leptos_use::use_resize_observer;
use web_sys::js_sys::Array;
use web_sys::{FillMode, ResizeObserverSize};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct SizeTransitionKeyframe {
    margin_right: String,
}

fn fuzzy_compare(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.1
}

/// Note: Only works for elements that infer their width from their contents;
/// Does not work for elements that infer their width from their container (like 1fr grid items).
#[component]
pub fn SizeTransition(children: Children) -> impl IntoView {
    #[allow(unused_variables)]
    let target_ref = NodeRef::<html::Span>::new();
    let last_width = StoredValue::new(None::<f64>);

    use_resize_observer(target_ref, move |entries, _| {
        let rects = entries[0].border_box_size();
        let rect: ResizeObserverSize = rects.get(0).into();
        let goal_width = rect.inline_size();

        let Some(current_width) = last_width.get_value() else {
            last_width.set_value(Some(goal_width));
            return;
        };

        last_width.set_value(Some(goal_width));

        let el = target_ref.get_untracked().unwrap();

        // Animate!

        let mut dynamics = SecondOrderDynamics::new(1.0, 0.8, 1.0, current_width);

        const ITERATION_RATE: f32 = 15.0;

        let mut values = vec![current_width];
        while !fuzzy_compare(dynamics.get(), goal_width) {
            dynamics.update(goal_width, 1.0 / ITERATION_RATE);
            values.push(dynamics.get().max(0.0));
            if values.len() > 1000 {
                logging::error!("Too many entries!");
                break;
            }
        }

        let duration = (values.len() - 1) as f64 / ITERATION_RATE as f64 * 1000.0;

        let arr: Array = values
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
            &duration.into(),
            FillMode::None,
            None,
        );
    });

    view! {
        <span style="display:inline-block" node_ref=target_ref>
            {children()}
        </span>
    }
}
