#[non_exhaustive]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[repr(i32)]
pub enum PenAxis {
    Unknown = -1,
    /// Pen pressure.  Unidirectional: 0 to 1.0
    Pressure = sys::pen::SDL_PEN_AXIS_PRESSURE.0,
    /// Pen horizontal tilt angle.  Bidirectional: -90.0 to 90.0 (left-to-right).
    XTilt = sys::pen::SDL_PEN_AXIS_XTILT.0,
    /// Pen vertical tilt angle.  Bidirectional: -90.0 to 90.0 (top-to-down).
    YTilt = sys::pen::SDL_PEN_AXIS_YTILT.0,
    /// Pen distance to drawing surface.  Unidirectional: 0.0 to 1.0
    Distance = sys::pen::SDL_PEN_AXIS_DISTANCE.0,
    /// Pen barrel rotation.  Bidirectional: -180 to 179.9 (clockwise, 0 is facing up, -180.0 is facing down).
    Rotation = sys::pen::SDL_PEN_AXIS_ROTATION.0,
    /// Pen finger wheel or slider (e.g., Airbrush Pen).  Unidirectional: 0 to 1.0
    Slider = sys::pen::SDL_PEN_AXIS_SLIDER.0,
    /// Pressure from squeezing the pen ("barrel pressure").
    TangentialPressure = sys::pen::SDL_PEN_AXIS_TANGENTIAL_PRESSURE.0,
    /// Total known pen axis types in this version of SDL. This number may grow in future releases!
    Count = sys::pen::SDL_PEN_AXIS_COUNT.0,
}

impl PenAxis {
    #[inline]
    pub fn from_ll(axis: sys::pen::SDL_PenAxis) -> PenAxis {
        match axis {
            sys::pen::SDL_PEN_AXIS_PRESSURE => PenAxis::Pressure,
            sys::pen::SDL_PEN_AXIS_XTILT => PenAxis::XTilt,
            sys::pen::SDL_PEN_AXIS_YTILT => PenAxis::YTilt,
            sys::pen::SDL_PEN_AXIS_DISTANCE => PenAxis::Distance,
            sys::pen::SDL_PEN_AXIS_ROTATION => PenAxis::Rotation,
            sys::pen::SDL_PEN_AXIS_SLIDER => PenAxis::Slider,
            sys::pen::SDL_PEN_AXIS_TANGENTIAL_PRESSURE => PenAxis::TangentialPressure,
            sys::pen::SDL_PEN_AXIS_COUNT => PenAxis::Count,
            _ => PenAxis::Unknown,
        }
    }
    #[inline]
    pub fn to_ll(self) -> sys::pen::SDL_PenAxis {
        sys::pen::SDL_PenAxis(self as i32)
    }
}
