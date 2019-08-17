use std::ops::{Index, IndexMut};

// A 2D matrix type, built row by row
#[derive(Debug)]
pub struct Matrix<T: Default> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

impl<T: Default> Matrix<T> {
    pub fn new(width: usize, expected_height: usize) -> Self {
        let data = Vec::with_capacity(width * expected_height);
        Matrix {
            width,
            height: 0,
            data,
        }
    }

    // Adds row filled with default values of type T
    pub fn add_row(&mut self) -> usize {
        let last_row = self.height;
        self.height += 1;
        self.data
            .resize_with(self.width * self.height, Default::default);
        last_row
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.data.get(x*self.width + y)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(x*self.width + y)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

// Enable indexing of Matrix with number tuples
impl<T: Default> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.data.index(x*self.width + y)
    }
}

impl<T: Default> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.data.index_mut(x*self.width + y)
    }
}
