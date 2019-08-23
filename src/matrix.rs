use std::ops::{Index, IndexMut};

// A 2D matrix type, built row by row
#[derive(Debug)]
pub struct Matrix<T> {
    width: usize,
    height: usize,
    data: Vec<T>,
}

pub struct MatrixRowIter<'a, T> {
    current: usize,
    matrix: &'a Matrix<T>,
}

impl<T: Default + Clone> Matrix<T> {
    pub fn new(width: usize, expected_height: usize) -> Self {
        let data = Vec::with_capacity(width * expected_height);
        Matrix {
            width,
            height: 0,
            data,
        }
    }

    pub fn new_fixed(width: usize, height: usize, element: T) -> Self {
        let data = vec![element; width * height];
        Matrix {
            width,
            height,
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
        self.data.get(x * self.width + y)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.data.get_mut(x * self.width + y)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn iter_rows<'a>(&'a self) -> MatrixRowIter<'a, T> {
        MatrixRowIter {
            current: 0,
            matrix: &self,
        }
    }
}

// Enable indexing of Matrix with number tuples
impl<T> Index<(usize, usize)> for Matrix<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        self.data.index(x * self.width + y)
    }
}

impl<T> IndexMut<(usize, usize)> for Matrix<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        self.data.index_mut(x * self.width + y)
    }
}

impl<'a, T> Iterator for MatrixRowIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.matrix.height {
            self.current = 0;
            None
        } else {
            self.current += 1;
            Some(
                &self.matrix.data
                    [(self.current - 1) * self.matrix.width..self.current * self.matrix.width],
            )
        }
    }
}
