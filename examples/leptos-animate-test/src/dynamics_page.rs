use leptos::prelude::*;
// use leptos_animate::dynamics::SecondOrderDynamics;
// use leptos_chartistry::{AspectRatio, AxisMarker, Chart, IntoInner, Series, TickLabels};

#[component]
pub fn DynamicsPage() -> impl IntoView {
    let frequency = RwSignal::new(0.5);
    let z = RwSignal::new(0.5);
    let r = RwSignal::new(0.0);

    // let data = Signal::derive(move || run_dynamics(frequency.get(), z.get(), r.get()));

    // let series = Series::new(|p: &DataPoint| p.x)
    //     .line(|p: &DataPoint| p.y)
    //     .line(|_: &DataPoint| 1.0);
    // let aspect_ratio = AspectRatio::from_outer_height(300.0, 1.2);

    let on_frequency_input = move |ev| {
        frequency.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let on_z_input = move |ev| {
        z.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let on_r_input = move |ev| {
        r.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    // let inner = [
    //     AxisMarker::left_edge().into_inner(),
    //     AxisMarker::bottom_edge().into_inner(),
    // ];

    view! {
        <div class="main-container dynamics-page">
            <div class="controls">
                <label>Frequency</label>
                <input
                    type="range" min="0.0" max="6.0" step="0.01"
                    prop:value=frequency on:input=on_frequency_input
                />
                <div>{frequency}</div>
                <label>Damping</label>
                <input
                    type="range" min="0.0" max="6.0" step="0.01"
                    prop:value=z on:input=on_z_input
                />
                <div>{z}</div>
                <label>Response</label>
                <input
                    type="range" min="-6.0" max="6.0" step="0.01"
                    prop:value=r on:input=on_r_input
                />
                <div>{r}</div>
            </div>
            // <Chart data series aspect_ratio left=TickLabels::aligned_floats() bottom=TickLabels::aligned_floats() inner />
        </div>
    }
}

// fn run_dynamics(f: f32, z: f32, r: f32) -> Vec<DataPoint> {
//     let mut dynamics = SecondOrderDynamics::new(f, z, r, 0.0);
//     let mut data = vec![];
//
//     const ITERATION_RATE: f32 = 15.0;
//     const DURATION: f32 = 2.0;
//
//     loop {
//         dynamics.update(1.0, 1.0 / ITERATION_RATE);
//         data.push(DataPoint {
//             x: data.len() as f64 / ITERATION_RATE as f64,
//             y: dynamics.get().max(-2.0).min(2.0),
//         });
//         if data.len() as f32 > ITERATION_RATE * DURATION {
//             break;
//         }
//     }
//
//     data
// }

// struct DataPoint {
//     x: f64,
//     y: f64,
// }
