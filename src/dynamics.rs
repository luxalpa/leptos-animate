pub trait DynamicValue: Copy + Default {
    fn scale(self, scale: f32) -> Self;
    fn add(self, other: Self) -> Self;
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

// https://www.youtube.com/watch?v=KPoeNZZ6H4s
pub struct SecondOrderDynamics<T>
where
    T: DynamicValue,
{
    goal: T,
    y: T,
    yd: T,
    k1: f32,
    k2: f32,
    k3: f32,
}

impl<T> SecondOrderDynamics<T>
where
    T: DynamicValue,
{
    /*
    f: frequency; response speed
    z: damping ratio, [0, 1] => damping after the end, 1+ => damping / delay before hitting the end
    r: gain at the start. 0 => start slowly, >1 => Overshoot, negative => anticipate
     */
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

    pub fn get(&self) -> T {
        self.y
    }
}
