use crate::color::Color;
use crate::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl CornerRadius {
    pub const ZERO: Self = Self {
        top_left: 0.0,
        top_right: 0.0,
        bottom_right: 0.0,
        bottom_left: 0.0,
    };

    pub fn uniform(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_right: radius,
            bottom_left: radius,
        }
    }

    pub fn is_uniform(&self) -> bool {
        self.top_left == self.top_right
            && self.top_right == self.bottom_right
            && self.bottom_right == self.bottom_left
    }
}

impl Default for CornerRadius {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Fill {
    Solid { color: Color, opacity: f32 },
}

impl Default for Fill {
    fn default() -> Self {
        Self::Solid {
            color: Color::WHITE,
            opacity: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeStyle {
    Solid,
    Dashed,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeAlign {
    Inside,
    Center,
    Outside,
}

impl Default for StrokeAlign {
    fn default() -> Self {
        Self::Inside
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
    pub align: StrokeAlign,
    pub opacity: f32,
    #[serde(default)]
    pub style: StrokeStyle,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            width: 1.0,
            align: StrokeAlign::Inside,
            opacity: 1.0,
            style: StrokeStyle::Solid,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    DropShadow {
        offset: Vec2,
        blur: f32,
        spread: f32,
        color: Color,
    },
    InnerShadow {
        offset: Vec2,
        blur: f32,
        spread: f32,
        color: Color,
    },
    LayerBlur {
        radius: f32,
    },
}
