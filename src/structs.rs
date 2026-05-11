use std::fmt::{Display, Formatter, Result};

struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    fn sum(&self, z: i32) -> i32 {
        if z < 0 {
            return 0;
        }

        self.x + self.y + z
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        "({}, {})".fmt(f, self.x, self.y)
    }
}

