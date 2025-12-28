use leptos::logging;
use leptos::prelude::*;
use leptos_animate::dynamics::SecondOrderDynamics;
use leptos_chartistry::{AspectRatio, AxisMarker, Chart, IntoInner, Series, TickLabels};

#[component]
pub fn DynamicsPage() -> impl IntoView {
    let frequency = RwSignal::new(2.0);
    let z = RwSignal::new(0.65);
    let r = RwSignal::new(0.0);
    let s = RwSignal::new(60u32);
    let iv = RwSignal::new(0.0);

    let data =
        Signal::derive(move || run_dynamics(frequency.get(), z.get(), r.get(), s.get(), iv.get()));

    let series = Series::new(|p: &DataPoint| p.x)
        .line(|p: &DataPoint| p.y)
        .line(|_: &DataPoint| 1.0);
    let aspect_ratio = AspectRatio::from_outer_height(600.0, 1.2);

    let on_frequency_input = move |ev| {
        frequency.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let on_z_input = move |ev| {
        z.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let on_r_input = move |ev| {
        r.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let on_s_input = move |ev| {
        s.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let on_iv_input = move |ev| {
        iv.set(event_target_value(&ev).parse().unwrap_or_default());
    };

    let inner = [
        AxisMarker::left_edge().into_inner(),
        AxisMarker::bottom_edge().into_inner(),
    ];

    let chart = view! {
        <Chart data series aspect_ratio left=TickLabels::aligned_floats() bottom=TickLabels::aligned_floats() inner />
    }.into_any();

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
                <label>Samples</label>
                <input
                    type="range" min="0" max="100" step="1"
                    prop:value=s on:input=on_s_input
                />
                <div>{s}</div>
                <label>Initial Velocity</label>
                <input
                    type="range" min="-10.0" max="10.0" step="0.1"
                    prop:value=iv on:input=on_iv_input
                />
                <div>{iv}</div>
            </div>
            {chart}
        </div>
    }
}

fn run_dynamics(
    f: f32,
    z: f32,
    r: f32,
    iteration_rate: u32,
    initial_velocity: f32,
) -> Vec<DataPoint> {
    let mut dynamics = SecondOrderDynamics::new(f, z, r, 0.0);
    let mut data = vec![];

    const DURATION: f32 = 2.0;

    let delta_time = 1.0 / iteration_rate as f32;

    dynamics.set_velocity((initial_velocity) as f64);

    loop {
        dynamics.update(1.0, delta_time);
        logging::log!("Velocity: {}", dynamics.velocity());
        data.push(DataPoint {
            x: data.len() as f64 / iteration_rate as f64,
            y: dynamics.get().max(-2.0).min(2.0),
        });
        if data.len() as f32 > (iteration_rate as f32) * DURATION {
            break;
        }
    }

    data
}

struct DataPoint {
    x: f64,
    y: f64,
}
