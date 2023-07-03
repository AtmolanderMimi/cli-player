use opencv::core::Mat;

/// Holds the contents of the Image
pub struct Image {
    content: Mat
}

impl Image {
    pub fn new(content: Mat) -> Image {
        Image {
            content
        }
    }
}

impl Image {
    pub fn content(&self) -> &Mat {
        &self.content
    }
}