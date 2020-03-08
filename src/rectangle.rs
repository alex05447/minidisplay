#![allow(non_upper_case_globals)]

/// 2D position of a point in display space.
/// Left-to-right, top-to-bottom.
/// Origin depends on context.
///
///  ------->
///  |
///  |
/// \/
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Position {
    pub left: i32,
    pub top: i32,
}

impl Default for Position {
    fn default() -> Self {
        Self { left: 0, top: 0 }
    }
}

impl Position {
    pub fn new(left: i32, top: i32) -> Self {
        Self { left, top }
    }
}

/// 2D dimensions of a rectangle in display space.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Default for Dimensions {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

impl Dimensions {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn area(self) -> u32 {
        self.width * self.height
    }
}

/// 2D rectangle in display space.
/// Left-to-right, top-to-bottom.
/// Origin depends on context.
///
///  ------->
///  |
///  |
/// \/
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rectangle {
    pub position: Position,
    pub dimensions: Dimensions,
}

impl Default for Rectangle {
    fn default() -> Self {
        Self {
            position: Position::default(),
            dimensions: Dimensions::default(),
        }
    }
}

bitflags! {
    pub struct ClipRectFlags: u32 {
        const KeepNone = 0;
        const KeepLeft = 1;
        const KeepRight = 1 << 1;
        const KeepTop = 1 << 2;
        const KeepBottom = 1 << 3;
    }
}

impl Rectangle {
    pub fn new(position: Position, dimensions: Dimensions) -> Self {
        Self {
            position,
            dimensions,
        }
    }

    pub fn left(&self) -> i32 {
        self.position.left
    }

    pub fn right(&self) -> i32 {
        self.position.left + self.dimensions.width as i32
    }

    pub fn top(&self) -> i32 {
        self.position.top
    }

    pub fn bottom(&self) -> i32 {
        self.position.top + self.dimensions.height as i32
    }

    pub fn width(&self) -> u32 {
        self.dimensions.width
    }

    pub fn height(&self) -> u32 {
        self.dimensions.height
    }

    /// Returns `true` if the rectangle overlaps the `other` rectangle.
    pub fn overlaps(&self, other: &Rectangle) -> bool {
        (self.left() < other.right())
            && (self.right() > other.left())
            && (self.top() < other.bottom())
            && (self.bottom() > other.top())
    }

    /// Returns `true` if the rectangle completely contains the `other` rectangle.
    pub fn contains(&self, other: &Rectangle) -> bool {
        (self.left() <= other.left())
            && (self.right() >= other.right())
            && (self.top() <= other.top())
            && (self.bottom() >= other.bottom())
    }

    /// Tries to clip the rectangle to the provided bounds.
    /// `clip_flags` control which sides of the rectangle to try to keep in place.
    /// Returns the clipped rectangle.
    pub fn clip(&self, clip_rect: &Rectangle, clip_flags: ClipRectFlags) -> Rectangle {
        // Clip to bottom and right sides.
        let mut right = self.right();
        let mut bottom = self.bottom();

        let furthest_right = clip_rect.right();
        right = at_most(right, furthest_right);

        let furthest_bottom = clip_rect.bottom();
        bottom = at_most(bottom, furthest_bottom);

        // Then clip to top and left.
        let left = if clip_flags.contains(ClipRectFlags::KeepLeft) {
            self.left()
        } else {
            right - self.width() as i32
        };
        debug_assert!(left <= self.left());

        let top = if clip_flags.contains(ClipRectFlags::KeepTop) {
            self.top()
        } else {
            bottom - self.height() as i32
        };
        debug_assert!(top <= self.top());

        let right = if clip_flags.contains(ClipRectFlags::KeepRight) {
            right
        } else {
            at_most(left + self.width() as i32, furthest_right)
        };

        let bottom = if clip_flags.contains(ClipRectFlags::KeepBottom) {
            bottom
        } else {
            at_most(top + self.height() as i32, furthest_bottom)
        };

        let position = Position::new(left, top);

        debug_assert!(right >= left);
        debug_assert!(bottom >= top);
        let dimensions = Dimensions::new((right - left) as u32, (bottom - top) as u32);

        Rectangle::new(position, dimensions)
    }

    /// Clamps the rectangle's dimensions to the provided minimum.
    /// Returns the clamped rectangle.
    pub fn clamp(&self, min_dimensions: Dimensions) -> Rectangle {
        let mut rectangle = *self;

        rectangle.dimensions.width = at_least(rectangle.dimensions.width, min_dimensions.width);
        rectangle.dimensions.height = at_least(rectangle.dimensions.height, min_dimensions.height);

        rectangle
    }
}

fn at_least<T: std::cmp::Ord>(val: T, min: T) -> T {
    val.max(min)
}

fn at_most<T: std::cmp::Ord>(val: T, max: T) -> T {
    val.min(max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlaps() {
        let rect_0 = Rectangle::new(Position::new(-1, -2), Dimensions::new(4, 3));

        assert!(rect_0.overlaps(&rect_0));

        let rect_1 = Rectangle::new(Position::new(1, -1), Dimensions::new(1, 4));

        assert!(rect_0.overlaps(&rect_1));
        assert!(rect_1.overlaps(&rect_0));

        let rect_2 = Rectangle::new(Position::new(-2, 0), Dimensions::new(1, 2));

        assert!(!rect_0.overlaps(&rect_2));
        assert!(!rect_2.overlaps(&rect_0));

        assert!(!rect_1.overlaps(&rect_2));
        assert!(!rect_2.overlaps(&rect_1));
    }

    #[test]
    fn contains() {
        let rect_0 = Rectangle::new(Position::new(-1, -2), Dimensions::new(4, 3));

        assert!(rect_0.contains(&rect_0));

        let rect_1 = Rectangle::new(Position::new(-1, -2), Dimensions::new(3, 2));

        assert!(rect_0.contains(&rect_1));
        assert!(!rect_1.contains(&rect_0));

        let rect_2 = Rectangle::new(Position::new(1, -1), Dimensions::new(1, 4));

        assert!(!rect_0.contains(&rect_2));
        assert!(!rect_2.contains(&rect_0));

        assert!(!rect_1.contains(&rect_2));
        assert!(!rect_2.contains(&rect_1));

        let rect_3 = Rectangle::new(Position::new(-2, 0), Dimensions::new(1, 2));

        assert!(!rect_0.contains(&rect_3));
        assert!(!rect_3.contains(&rect_0));

        assert!(!rect_1.contains(&rect_3));
        assert!(!rect_3.contains(&rect_1));

        assert!(!rect_2.contains(&rect_3));
        assert!(!rect_3.contains(&rect_2));
    }
}
