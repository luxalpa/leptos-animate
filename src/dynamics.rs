/// Trait for any value to be used in dynamics. Note: Does not work for rotations, which need a
/// slightly different dynamics implementation.
pub trait DynamicValue: Copy + Default {
    /// operation to handle multiplication, like scaling a vector by a constant.
    fn scale(self, scale: f32) -> Self;

    /// Addition: `self + other`.
    fn add(self, other: Self) -> Self;

    /// Subtraction: `self - other`
    fn sub(self, other: Self) -> Self;
}

impl DynamicValue for f64 {
    fn scale(self, scale: f32) -> Self {
        self * scale as f64
    }

    fn add(self, other: Self) -> Self {
        self + other
    }

    fn sub(self, other: Self) -> Self {
        self - other
    }
}

/// Second order dynamics simulation.
/// <https://www.youtube.com/watch?v=KPoeNZZ6H4s>
pub struct SecondOrderDynamics<T>
where
    T: DynamicValue,
{
    /// The value we're currently trying to reach
    goal: T,

    /// The current value
    y: T,

    /// The current velocity
    yd: T,

    /// Constant
    k1: f32,
    /// Constant
    k2: f32,
    /// Constant
    k3: f32,
}

impl<T> SecondOrderDynamics<T>
where
    T: DynamicValue,
{
    /// Create and initiate a new dynamics simulation.
    /// f: frequency; response speed
    /// z: damping ratio, [0, 1] => damping after the end, 1+ => damping / delay before hitting the end
    /// r: gain at the start. 0 => start slowly, >1 => Overshoot, negative => anticipate
    pub fn new(f: f32, z: f32, r: f32, x0: T) -> Self {
        use std::f32::consts::PI;

        SecondOrderDynamics {
            goal: x0,
            y: x0,
            yd: T::default(),
            k1: z / (PI * f),
            k2: 1.0 / ((2.0 * PI * f) * (2.0 * PI * f)),
            k3: r * z / (2.0 * PI * f),
        }
    }

    /// Step the dynamics simulation to try to reach `new_goal` in the timestep `dt`.
    pub fn update(&mut self, new_goal: T, dt: f32) {
        let xd = new_goal.sub(self.goal).scale(1.0 / dt);
        self.goal = new_goal;
        self.y = self.y.add(self.yd.scale(dt));
        self.yd = new_goal
            .add(xd.scale(self.k3))
            .sub(self.y)
            .sub(self.yd.scale(self.k1))
            .scale(dt / self.k2)
            .add(self.yd);
    }

    /// Get the current position of the simulated value.
    pub fn get(&self) -> T {
        self.y
    }

    /// Get the current velocity of the simulated value. Useful for checking if the simulation has
    /// converged.
    pub fn velocity(&self) -> T {
        self.yd
    }
}
