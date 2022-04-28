#[derive(Clone, Debug)]
pub struct Window {
    pub x_width: usize,
    pub y_height: usize,
}

impl Window {
    pub fn get_x(&self) -> usize {
        self.x_width
    }

    pub fn get_y(&self) -> usize {
        self.y_height
    }
}
