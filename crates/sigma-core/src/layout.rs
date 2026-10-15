use crate::math::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}

impl Default for LayoutDirection {
    fn default() -> Self {
        Self::Horizontal
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutAlign {
    Start,
    Center,
    End,
    SpaceBetween,
}

impl Default for LayoutAlign {
    fn default() -> Self {
        Self::Start
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SizingMode {
    Fixed(f32),
    Hug,
    Fill,
}

impl Default for SizingMode {
    fn default() -> Self {
        Self::Hug
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Padding {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Padding {
    pub const ZERO: Self = Self {
        top: 0.0,
        right: 0.0,
        bottom: 0.0,
        left: 0.0,
    };

    pub fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AutoLayout {
    pub enabled: bool,
    pub direction: LayoutDirection,
    pub main_axis_align: LayoutAlign,
    pub cross_axis_align: LayoutAlign,
    pub gap: f32,
    pub padding: Padding,
}

impl Default for AutoLayout {
    fn default() -> Self {
        Self {
            enabled: false,
            direction: LayoutDirection::Horizontal,
            main_axis_align: LayoutAlign::Start,
            cross_axis_align: LayoutAlign::Center,
            gap: 8.0,
            padding: Padding::all(16.0),
        }
    }
}

impl AutoLayout {
    /// Computes the positions of children relative to the frame origin
    /// and optionally returns the fitted content size (for Hug sizing).
    pub fn compute_children_positions(
        &self,
        frame_size: Vec2,
        child_sizes: &[Vec2],
    ) -> (Vec<Vec2>, Vec2) {
        if child_sizes.is_empty() {
            let hug_w = self.padding.left + self.padding.right;
            let hug_h = self.padding.top + self.padding.bottom;
            return (vec![], Vec2::new(hug_w, hug_h));
        }

        let mut positions = Vec::with_capacity(child_sizes.len());
        match self.direction {
            LayoutDirection::Horizontal => {
                let total_gap = (child_sizes.len() - 1) as f32 * self.gap;
                let children_width: f32 = child_sizes.iter().map(|s| s.x).sum();
                let max_child_h: f32 = child_sizes.iter().map(|s| s.y).fold(0.0, f32::max);

                let available_w = frame_size.x - self.padding.left - self.padding.right;
                let inner_h = frame_size.y - self.padding.top - self.padding.bottom;

                let start_x = match self.main_axis_align {
                    LayoutAlign::Start => self.padding.left,
                    LayoutAlign::Center => {
                        self.padding.left + ((available_w - (children_width + total_gap)) * 0.5).max(0.0)
                    }
                    LayoutAlign::End => {
                        self.padding.left + (available_w - (children_width + total_gap)).max(0.0)
                    }
                    LayoutAlign::SpaceBetween => self.padding.left,
                };

                let effective_gap = if self.main_axis_align == LayoutAlign::SpaceBetween && child_sizes.len() > 1 {
                    (available_w - children_width) / (child_sizes.len() - 1) as f32
                } else {
                    self.gap
                };

                let mut current_x = start_x;
                for size in child_sizes {
                    let y = match self.cross_axis_align {
                        LayoutAlign::Start => self.padding.top,
                        LayoutAlign::Center => {
                            self.padding.top + ((inner_h - size.y) * 0.5).max(0.0)
                        }
                        LayoutAlign::End => self.padding.top + (inner_h - size.y).max(0.0),
                        LayoutAlign::SpaceBetween => self.padding.top,
                    };
                    positions.push(Vec2::new(current_x, y));
                    current_x += size.x + effective_gap;
                }

                let hug_size = Vec2::new(
                    children_width + total_gap + self.padding.left + self.padding.right,
                    max_child_h + self.padding.top + self.padding.bottom,
                );
                (positions, hug_size)
            }
            LayoutDirection::Vertical => {
                let total_gap = (child_sizes.len() - 1) as f32 * self.gap;
                let children_height: f32 = child_sizes.iter().map(|s| s.y).sum();
                let max_child_w: f32 = child_sizes.iter().map(|s| s.x).fold(0.0, f32::max);

                let available_h = frame_size.y - self.padding.top - self.padding.bottom;
                let inner_w = frame_size.x - self.padding.left - self.padding.right;

                let start_y = match self.main_axis_align {
                    LayoutAlign::Start => self.padding.top,
                    LayoutAlign::Center => {
                        self.padding.top + ((available_h - (children_height + total_gap)) * 0.5).max(0.0)
                    }
                    LayoutAlign::End => {
                        self.padding.top + (available_h - (children_height + total_gap)).max(0.0)
                    }
                    LayoutAlign::SpaceBetween => self.padding.top,
                };

                let effective_gap = if self.main_axis_align == LayoutAlign::SpaceBetween && child_sizes.len() > 1 {
                    (available_h - children_height) / (child_sizes.len() - 1) as f32
                } else {
                    self.gap
                };

                let mut current_y = start_y;
                for size in child_sizes {
                    let x = match self.cross_axis_align {
                        LayoutAlign::Start => self.padding.left,
                        LayoutAlign::Center => {
                            self.padding.left + ((inner_w - size.x) * 0.5).max(0.0)
                        }
                        LayoutAlign::End => self.padding.left + (inner_w - size.x).max(0.0),
                        LayoutAlign::SpaceBetween => self.padding.left,
                    };
                    positions.push(Vec2::new(x, current_y));
                    current_y += size.y + effective_gap;
                }

                let hug_size = Vec2::new(
                    max_child_w + self.padding.left + self.padding.right,
                    children_height + total_gap + self.padding.top + self.padding.bottom,
                );
                (positions, hug_size)
            }
        }
    }
}
