#[derive(Default, Clone, Copy)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from_points((x0, y0): (f32, f32), (x1, y1): (f32, f32)) -> Self {
        let (x, y, w, h) = (x0.min(x1), y0.min(y1), (x0 - x1).abs(), (y0 - y1).abs());
        Self::new(x, y, w, h)
    }

    pub fn from_center_size(x: f32, y: f32, size: f32) -> Self {
        Self::new(x - size / 2.0, y - size / 2.0, size, size)
    }
}

impl<T> From<(T, T, T, T)> for Rectangle
where
    T: Into<f32>,
{
    fn from((x, y, width, height): (T, T, T, T)) -> Self {
        Self::new(x.into(), y.into(), width.into(), height.into())
    }
}

impl<T> From<((T, T), (T, T))> for Rectangle
where
    T: Into<f32>,
{
    fn from(((x0, y0), (x1, y1)): ((T, T), (T, T))) -> Self {
        Self::from_points((x0.into(), y0.into()), (x1.into(), y1.into()))
    }
}
