use crate::{Dimensions, Position, Rectangle};

use winapi::shared::windef::RECT;

impl Rectangle {
    pub fn from_win_rect(rect: &RECT) -> Rectangle {
        assert!(rect.right >= rect.left, "Negative width Windows rectangle.");
        assert!(
            rect.bottom >= rect.top,
            "Negative height Windows rectangle."
        );

        Rectangle::new(
            Position::new(rect.left, rect.top),
            Dimensions::new(
                (rect.right - rect.left) as u32,
                (rect.bottom - rect.top) as u32,
            ),
        )
    }

    pub fn to_win_rect(&self) -> RECT {
        RECT {
            left: self.left(),
            top: self.top(),
            right: self.right(),
            bottom: self.bottom(),
        }
    }
}
