use std::{
    fmt::Display,
    ops::{Add, AddAssign},
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub x: usize,
    pub y: usize,
}

impl Pos {
    pub fn new(x: usize, y: usize) -> Pos {
        Pos { x, y }
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl Add for Pos {
    type Output = Pos;

    fn add(self, rhs: Self) -> Self::Output {
        Pos {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl AddAssign for Pos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add<(usize, usize)> for Pos {
    type Output = Pos;

    fn add(self, rhs: (usize, usize)) -> Self::Output {
        self + Pos::from(rhs)
    }
}

impl AddAssign<(usize, usize)> for Pos {
    fn add_assign(&mut self, rhs: (usize, usize)) {
        *self += Pos::from(rhs)
    }
}

impl From<(usize, usize)> for Pos {
    fn from(value: (usize, usize)) -> Self {
        Pos::new(value.0, value.1)
    }
}
