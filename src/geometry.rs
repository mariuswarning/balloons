use crate::ball::is_point_in_rect;
use crate::geometry::Cells::{BottomLeft, BottomRight, TopLeft, TopRight};
use crate::random::{random_range, random_sign, random_velocity};
use std::fmt;

pub enum Cells {
    TopLeft = 1,
    TopRight = 2,
    BottomLeft = 3,
    BottomRight = 4,
}

#[derive(Debug, Copy, Clone)]
pub struct RenderingRect {
    pub rect: Rect,
    pub many: bool,
}

#[derive(Debug, Copy, Clone)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    // pub fn intersects(&self, other: Rect) -> bool {
    //     let top_left = Point {
    //         x: self.x,
    //         y: self.y,
    //     }
    //     .is_in_rect(&other);
    //     let top_right = Point {
    //         x: self.x + self.w,
    //         y: self.y,
    //     }
    //     .is_in_rect(&other);
    //     let bottom_left = Point {
    //         x: self.x,
    //         y: self.y + self.h,
    //     }
    //     .is_in_rect(&other);
    //     let bottom_right = Point {
    //         x: self.x + self.w,
    //         y: self.y + self.h,
    //     }
    //     .is_in_rect(&other);
    //
    //     if top_left || top_right || bottom_left || bottom_right {
    //         return true;
    //     }
    //
    //     false
    // }

    /// Checks if the current rectangle intersects with another rectangle.
    pub fn intersects(&self, other: &Rect) -> bool {
        // Early return if any corners of self are within 'other'.
        if (Point { x: self.x, y: self.y }.is_in_rect(other) ||
            Point { x: self.x + self.w, y: self.y }.is_in_rect(other) ||
            Point { x: self.x, y: self.y + self.h }.is_in_rect(other) ||
            Point { x: self.x + self.w, y: self.y + self.h }.is_in_rect(other)) {
            return true;
        }

        // Early return if any corners of 'other' are within self.
        if (Point { x: other.x, y: other.y }.is_in_rect(self) ||
            Point { x: other.x + other.w, y: other.y }.is_in_rect(self) ||
            Point { x: other.x, y: other.y + other.h }.is_in_rect(self) ||
            Point { x: other.x + other.w, y: other.y + other.h }.is_in_rect(self)) {
            return true;
        }

        // Check if 'other' overlaps 'self' horizontally or vertically.
        self.x < other.x + other.w && self.x + self.w > other.x &&
            self.y < other.y + other.h && self.y + self.h > other.y
    }

    pub fn fits(&self, other: Rect) -> bool {
        let top_left = Point {
            x: self.x,
            y: self.y,
        }
        .is_in_rect(&other);
        let top_right = Point {
            x: self.x + self.w,
            y: self.y,
        }
        .is_in_rect(&other);
        let bottom_left = Point {
            x: self.x,
            y: self.y - self.h,
        }
        .is_in_rect(&other);
        let bottom_right = Point {
            x: self.x + self.w,
            y: self.y - self.h,
        }
        .is_in_rect(&other);

        if top_left && top_right && bottom_left && bottom_right {
            return true;
        }

        false
    }

    pub fn split_to_four_cells(&self) -> (Rect, Rect, Rect, Rect) {
        (
            self.top_left(),
            self.top_right(),
            self.bottom_left(),
            self.bottom_right(),
        )

        // let new_w = self.w / 2.0;
        // let new_h = self.h / 2.0;
        //
        // let top_left = Rect { x: self.x, y: self.y, w: new_w, h: new_h };
        // let top_right = Rect { x: self.x + new_w, y: self.y, w: new_w, h: new_h };
        // let bottom_left = Rect { x: self.x, y: self.y + new_h, w: new_w, h: new_h };
        // let bottom_right = Rect { x: self.x + new_w, y: self.y + new_h, w: new_w, h: new_h };
        //
        // return (top_left, top_right, bottom_left, bottom_right);
    }

    /// Returns the center point of the rectangle.
    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.w / 2.0,
            y: self.y + self.h / 2.0,
        }
    }

    pub fn where_is_point_relative_to_center(self, point: Point) -> Cells {
        let center = self.center();

        match (point.x > center.x, point.y > center.y) {
            (true, true) => BottomRight,
            (true, false) => TopRight,
            (false, true) => BottomLeft,
            (false, false) => TopLeft,
        }

        // return if center.x < point.x {
        //     if center.y < point.y {
        //         BottomRight
        //     } else {
        //         TopRight
        //     }
        // } else if center.y < point.y {
        //     BottomLeft
        // } else {
        //     TopLeft
        // }
    }

    fn half_dimensions(&self) -> (f64, f64) {
        (self.w / 2.0, self.h / 2.0)
    }

    /// Returns the top-left quarter of the current rectangle.
    pub fn top_left(&self) -> Rect {
        let (half_width, half_height) = self.half_dimensions();
        Rect {
            x: self.x,
            y: self.y,
            w: half_width,
            h: half_height,
        }
    }

    /// Returns the top-right quarter of the current rectangle.
    pub fn top_right(&self) -> Rect {
        let (half_width, half_height) = self.half_dimensions();
        Rect {
            x: self.x + half_width,
            y: self.y,
            w: half_width,
            h: half_height,
        }
    }

    /// Returns the bottom-left quarter of the current rectangle.
    pub fn bottom_left(&self) -> Rect {
        let (half_width, half_height) = self.half_dimensions();
        Rect {
            x: self.x,
            y: self.y + half_height,
            w: half_width,
            h: half_height,
        }
    }

    /// Returns the bottom-right quarter of the current rectangle.
    pub fn bottom_right(&self) -> Rect {
        let (half_width, half_height) = self.half_dimensions();
        Rect {
            x: self.x + half_width,
            y: self.y + half_height,
            w: half_width,
            h: half_height,
        }
    }

    // pub fn top_left(&self) -> Rect {
    //     let new_w = self.w / 2.0;
    //     let new_h = self.h / 2.0;
    //     Rect { x: self.x, y: self.y, w: new_w, h: new_h }
    // }
    //
    // pub fn top_right(&self) -> Rect {
    //     let new_w = self.w / 2.0;
    //     let new_h = self.h / 2.0;
    //     Rect { x: self.x + new_w, y: self.y, w: new_w, h: new_h }
    // }
    //
    // pub fn bottom_left(&self) -> Rect {
    //     let new_w = self.w / 2.0;
    //     let new_h = self.h / 2.0;
    //
    //     Rect { x: self.x, y: self.y + new_h, w: new_w, h: new_h }
    // }
    //
    // pub fn bottom_right(&self) -> Rect {
    //     let new_w = self.w / 2.0;
    //     let new_h = self.h / 2.0;
    //
    //     Rect { x: self.x + new_w, y: self.y + new_h, w: new_w, h: new_h }
    // }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <{}>", self.x, self.y)
    }
}

impl Point {
    pub fn random_point(width: usize, height: usize) -> Self {
        Self {
            x: random_range(width / 10, 9 * width / 10) as f64,
            y: random_range(width / 10, 9 * height / 10) as f64,
        }
    }
    pub fn random_velocity(min: f64, max: f64) -> Self {
        Self {
            x: random_sign() * random_velocity(min, max),
            y: random_sign() * random_velocity(min, max),
        }
    }

    pub fn is_in_rect(&self, rect: &Rect) -> bool {
        return is_point_in_rect(rect.x, rect.y, rect.w, rect.h, self.x, self.y);
    }

    // fn is_point_in_rect(x: f64, y: f64, w: f64, h: f64, point_x: f64, point_y: f64) -> bool {
    //     if point_x > x
    //         && point_x < x + w
    //         && point_y > y
    //         && point_y < y + h {
    //         return true;
    //     }
    //     false
    // }

    /// Checks if a point is inside the given rectangle.
    fn is_point_in_rect(x: f64, y: f64, w: f64, h: f64, point_x: f64, point_y: f64) -> bool {
        point_x > x && point_x < x + w && point_y > y && point_y < y + h
    }
}
