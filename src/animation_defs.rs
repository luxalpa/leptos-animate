use crate::Position;
use leptos::Oco;
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

pub trait EnterAnimation {
    type Props: serde::Serialize;
    fn enter(&self) -> AnimationConfig<Self::Props>;
}

pub trait LeaveAnimation {
    type Props: serde::Serialize;
    fn leave(&self) -> AnimationConfig<Self::Props>;
}

pub trait MoveAnimation {
    type Props: serde::Serialize;
    fn animate(&self, from: Position, to: Position) -> AnimationConfigMove;
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
    type Props = ();

    fn animate(&self, _from: Position, _to: Position) -> AnimationConfigMove {
        let duration = self.duration;
        let timing_fn = Some(self.timing_fn.clone());

        AnimationConfigMove {
            duration,
            timing_fn,
        }
    }
}
