use crate::{ElementSnapshot, Extent, SecondOrderDynamics};
use itertools::Itertools;
use leptos::{logging, Oco};
use std::time::Duration;

pub struct AnimationConfig<T: serde::Serialize> {
    pub duration: Duration,
    pub timing_fn: Option<Oco<'static, str>>,
    pub keyframes: Vec<T>,
}

pub struct AnimationConfigMove {
    pub duration: Duration,
    pub timing_fn: Option<Oco<'static, str>>,
}

pub struct AnimationConfigResize {
    pub duration: Duration,
    pub timing_fn: Option<Oco<'static, str>>,
}

pub trait EnterAnimation {
    type Props: serde::Serialize;
    fn enter(&self) -> AnimationConfig<Self::Props>;
}

pub trait LeaveAnimation {
    type Props: serde::Serialize;
    fn leave(&self) -> AnimationConfig<Self::Props>;
}

pub trait MoveAnimation {
    // type Props: serde::Serialize;
    fn animate(&self, from: ElementSnapshot, to: ElementSnapshot) -> AnimationConfigMove;
}

pub trait ResizeAnimation {
    fn animate(&self, from: Extent, to: Extent) -> AnimationConfigResize;
}

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

fn fuzzy_compare(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.01
}

pub struct DynamicsAnimation {
    timing_fn: Oco<'static, str>,
    duration: Duration,
}

impl DynamicsAnimation {
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
