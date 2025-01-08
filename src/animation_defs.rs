use crate::{dynamics::SecondOrderDynamics, ElementSnapshot, Extent};
use itertools::Itertools;
use leptos::logging;
use leptos::oco::Oco;
use std::time::Duration;

/// Return value for any enter/leave animation.
pub struct AnimationConfig<T: serde::Serialize> {
    /// Duration of the animation
    pub duration: Duration,

    /// Timing function of the animation (passed as the [`easing` parameter](https://developer.mozilla.org/en-US/docs/Web/API/KeyframeEffect/KeyframeEffect#easing) to JS)
    pub timing_fn: Option<Oco<'static, str>>,

    /// Keyframes. Ensure that `T` uses `#[serde(rename_all = "camelCase")]`
    pub keyframes: Vec<T>,
}

/// Return value for any move animation.
pub struct AnimationConfigMove {
    /// Duration of the animation
    pub duration: Duration,

    /// Timing function of the animation (passed as the [`easing` parameter](https://developer.mozilla.org/en-US/docs/Web/API/KeyframeEffect/KeyframeEffect#easing) to JS)
    pub timing_fn: Option<Oco<'static, str>>,
}

/// Return value for any resize animation - currently only used in [`SizeTransition`][crate::SizeTransition].
pub struct AnimationConfigResize {
    /// Duration of the animation
    pub duration: Duration,

    /// Timing function of the animation (passed as the [`easing` parameter](https://developer.mozilla.org/en-US/docs/Web/API/KeyframeEffect/KeyframeEffect#easing) to JS)
    pub timing_fn: Option<Oco<'static, str>>,
}

/// Trait for defining an enter animation.
pub trait EnterAnimation {
    /// The CSS properties on the keyframes.
    type Props: serde::Serialize;

    /// Generate the keyframes, timing function, duration, etc.
    fn enter(&self) -> AnimationConfig<Self::Props>;
}

/// Trait for defining a leave animation.
pub trait LeaveAnimation {
    /// The CSS properties on the keyframes.
    type Props: serde::Serialize;

    /// Generate the keyframes, timing function, duration, etc.
    fn leave(&self) -> AnimationConfig<Self::Props>;
}

/// Trait for defining a move animation.
pub trait MoveAnimation {
    // type Props: serde::Serialize;

    /// Generate the timing function and duration. Currently does not support keyframes.
    /// The `from` and `to` parameters are not useful currently. Also, `ElementSnapshot::extent`
    /// will be 0 if `animate_size` is not set on the [`AnimatedFor`][crate::AnimatedFor].
    fn animate(&self, from: ElementSnapshot, to: ElementSnapshot) -> AnimationConfigMove;
}

/// Trait for defining a resize animation (currently only used in [`SizeTransition`][crate::SizeTransition]).
pub trait ResizeAnimation {
    /// Generate the timing function and duration. Currently does not support keyframes which makes
    /// the `from` and `to` parameters not very useful.
    fn animate(&self, from: Extent, to: Extent) -> AnimationConfigResize;
}

/// A simple enter / leave animation that fades the elements in and out using `opacity`
pub struct FadeAnimation {
    pub timing_fn: Oco<'static, str>,
    pub duration: Duration,
}

impl FadeAnimation {
    pub fn new<TF: Into<Oco<'static, str>>>(duration: Duration, timing_fn: TF) -> Self {
        Self {
            duration,
            timing_fn: timing_fn.into(),
        }
    }
}

impl Default for FadeAnimation {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
            timing_fn: Oco::Borrowed("ease-out"),
        }
    }
}

#[doc(hidden)]
#[derive(serde::Serialize)]
pub struct FadeAnimationProps {
    opacity: f64,
}

impl EnterAnimation for FadeAnimation {
    type Props = FadeAnimationProps;

    fn enter(&self) -> AnimationConfig<Self::Props> {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfig {
            duration,
            timing_fn,
            keyframes: vec![
                FadeAnimationProps { opacity: 0.0 },
                FadeAnimationProps { opacity: 1.0 },
            ],
        }
    }
}

impl LeaveAnimation for FadeAnimation {
    type Props = FadeAnimationProps;

    fn leave(&self) -> AnimationConfig<Self::Props> {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfig {
            duration,
            timing_fn,
            keyframes: vec![
                FadeAnimationProps { opacity: 1.0 },
                FadeAnimationProps { opacity: 0.0 },
            ],
        }
    }
}

/// A simple move / resize animation that changes the respective props based on the timing function.
pub struct SlidingAnimation {
    pub timing_fn: Oco<'static, str>,
    pub duration: Duration,
}

impl Default for SlidingAnimation {
    fn default() -> Self {
        Self {
            timing_fn: Oco::Borrowed("ease-out"),
            duration: Duration::from_millis(200),
        }
    }
}

impl SlidingAnimation {
    pub fn new<TF: Into<Oco<'static, str>>>(duration: Duration, timing_fn: TF) -> Self {
        Self {
            duration,
            timing_fn: timing_fn.into(),
        }
    }
}

impl MoveAnimation for SlidingAnimation {
    fn animate(&self, _from: ElementSnapshot, _to: ElementSnapshot) -> AnimationConfigMove {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfigMove {
            duration,
            timing_fn,
        }
    }
}

impl ResizeAnimation for SlidingAnimation {
    fn animate(&self, _from: Extent, _to: Extent) -> AnimationConfigResize {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfigResize {
            duration,
            timing_fn,
        }
    }
}

/// Comparison for checking if velocity on the simulation has converged.
fn fuzzy_compare(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

/// A move / resize animation using a simulation of [second order dynamics](https://www.youtube.com/watch?v=KPoeNZZ6H4s).
pub struct DynamicsAnimation {
    timing_fn: Oco<'static, str>,
    duration: Duration,
}

impl DynamicsAnimation {
    /// Create and initiate a new dynamics simulation.
    ///
    /// f: frequency; response speed
    /// z: damping ratio, [0, 1] => damping after the end, 1+ => damping / delay before hitting the end
    /// r: gain at the start. 0 => start slowly, >1 => Overshoot, negative => anticipate
    pub fn new(f: f32, z: f32, r: f32) -> Self {
        let mut dynamics = SecondOrderDynamics::new(f, z, r, 0.0);
        let mut data = vec![];

        const ITERATION_RATE: f32 = 15.0;

        loop {
            dynamics.update(1.0, 1.0 / ITERATION_RATE);
            data.push(dynamics.get());
            if data.len() > 1000 {
                logging::error!("DynamicsAnimation too long!");
                break;
            }

            if fuzzy_compare(dynamics.velocity(), 0.0) {
                break;
            }
        }

        let duration = Duration::from_secs_f32(data.len() as f32 / ITERATION_RATE);

        Self {
            duration,
            timing_fn: Oco::Owned(format!("linear({})", data.iter().join(", "))),
        }
    }
}

impl MoveAnimation for DynamicsAnimation {
    fn animate(&self, _from: ElementSnapshot, _to: ElementSnapshot) -> AnimationConfigMove {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfigMove {
            duration,
            timing_fn,
        }
    }
}

impl ResizeAnimation for DynamicsAnimation {
    fn animate(&self, _from: Extent, _to: Extent) -> AnimationConfigResize {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfigResize {
            duration,
            timing_fn,
        }
    }
}
