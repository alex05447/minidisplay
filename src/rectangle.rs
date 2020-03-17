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
    /// Flags which specify the sides of the rectangle to (attempt to) keep in place when clipping / clamping.
    pub struct ClipRectFlags: u32 {
        /// Move the rectangle as appropriate.
        const KeepNone = 0;
        /// Do not move the left side of the rectangle.
        const KeepLeft = 1;
        /// Do not move the right side of the rectangle.
        const KeepRight = 1 << 1;
        /// Do not move the top side of the rectangle.
        const KeepTop = 1 << 2;
        /// Do not move the bottom side of the rectangle.
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
        // Clip to bottom and right sides, finding top and left coordinates.
        let mut right = self.right();
        let mut bottom = self.bottom();

        let furthest_right = clip_rect.right();
        right = at_most(right, furthest_right);

        let furthest_bottom = clip_rect.bottom();
        bottom = at_most(bottom, furthest_bottom);

        let mut left = if clip_flags.contains(ClipRectFlags::KeepLeft) {
            self.left()
        } else {
            right - self.width() as i32
        };
        debug_assert!(left <= self.left());
        left = at_least(left, clip_rect.left());

        let mut top = if clip_flags.contains(ClipRectFlags::KeepTop) {
            self.top()
        } else {
            bottom - self.height() as i32
        };
        debug_assert!(top <= self.top());
        top = at_least(top, clip_rect.top());

        // Then clip to top and left, finding the bottom and right coordinates.
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
    /// `clip_flags` control which sides of the rectangle to keep in place.
    /// Returns the clamped rectangle.
    pub fn clamp(&self, min_dimensions: Dimensions, clip_flags: ClipRectFlags) -> Rectangle {
        let left = self.left();
        let top = self.top();

        let width = at_least(self.width(), min_dimensions.width);
        let height = at_least(self.height(), min_dimensions.height);

        let right = if clip_flags.contains(ClipRectFlags::KeepRight) {
            self.right()
        } else {
            left + width as i32
        };

        let bottom = if clip_flags.contains(ClipRectFlags::KeepBottom) {
            self.bottom()
        } else {
            top + height as i32
        };

        let left = if clip_flags.contains(ClipRectFlags::KeepLeft) {
            self.left()
        } else {
            right - width as i32
        };

        let top = if clip_flags.contains(ClipRectFlags::KeepTop) {
            self.top()
        } else {
            bottom - height as i32
        };

        debug_assert!(right >= (left + min_dimensions.width as i32));
        debug_assert!(bottom >= (top + min_dimensions.height as i32));

        let width = (right - left) as u32;
        let height = (bottom - top) as u32;

        Rectangle {
            position: Position::new(left, top),
            dimensions: Dimensions::new(
                width,
                height,
            )
        }
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

    #[test]
    fn clamp() {
        let min_dimensions = Dimensions::new(3, 2);

        // Resizing on the left.
        let rect = Rectangle::new(Position::new(-1, -2), Dimensions::new(2, 3));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepRight), Rectangle::new(Position::new(-2, -2), Dimensions::new(3, 3)));

        // Resizing on the right.
        let rect = Rectangle::new(Position::new(-3, -2), Dimensions::new(2, 3));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepLeft), Rectangle::new(Position::new(-3, -2), Dimensions::new(3, 3)));

        // Resizing on the top.
        let rect = Rectangle::new(Position::new(-3, 0), Dimensions::new(4, 1));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepBottom), Rectangle::new(Position::new(-3, -1), Dimensions::new(4, 2)));

        // Resizing on the bottom.
        let rect = Rectangle::new(Position::new(-3, -2), Dimensions::new(4, 1));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepTop), Rectangle::new(Position::new(-3, -2), Dimensions::new(4, 2)));

        // Resizing on the left and top.
        let rect = Rectangle::new(Position::new(0, 0), Dimensions::new(1, 1));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepRight | ClipRectFlags::KeepBottom), Rectangle::new(Position::new(-2, -1), Dimensions::new(3, 2)));

        // Resizing on the left and bottom.
        let rect = Rectangle::new(Position::new(0, -2), Dimensions::new(1, 1));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepRight | ClipRectFlags::KeepTop), Rectangle::new(Position::new(-2, -2), Dimensions::new(3, 2)));

        // Resizing on the right and bottom.
        let rect = Rectangle::new(Position::new(-3, -2), Dimensions::new(1, 1));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepLeft | ClipRectFlags::KeepTop), Rectangle::new(Position::new(-3, -2), Dimensions::new(3, 2)));

        // Resizing on the right and top.
        let rect = Rectangle::new(Position::new(-3, 0), Dimensions::new(1, 1));
        assert_eq!(rect.clamp(min_dimensions, ClipRectFlags::KeepLeft | ClipRectFlags::KeepBottom), Rectangle::new(Position::new(-3, -1), Dimensions::new(3, 2)));
    }
}
