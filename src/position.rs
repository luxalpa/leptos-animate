#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

fn fuzzy_compare(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.1
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        fuzzy_compare(self.x, other.x) && fuzzy_compare(self.y, other.y)
    }
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl std::ops::Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl From<(f64, f64)> for Position {
    fn from((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Extent {
    pub width: f64,
    pub height: f64,
}

impl From<(f64, f64)> for Extent {
    fn from((width, height): (f64, f64)) -> Self {
        Self { width, height }
    }
}

impl PartialEq for Extent {
    fn eq(&self, other: &Self) -> bool {
        fuzzy_compare(self.width, other.width) && fuzzy_compare(self.height, other.height)
    }
}
