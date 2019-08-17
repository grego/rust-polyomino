use crate::matrix::Matrix;
use crate::tiles::Point;

pub struct Image {
    pub data: Matrix<Option<usize>>,
    pub points: Vec<Point>,
}

impl Image {
    pub fn load(reader: impl std::io::BufRead, filled: char) -> Self {
        let mut points = Vec::new();
        let mut data = Matrix::new(255, 255);
        for (line, x) in reader
            .lines()
            .filter_map(Result::ok)
            .take_while(|s| !s.is_empty())
            .zip(0..=255)
        // iterate with x coordinate
        {
            data.add_row();
            for (_, y) in line
                .chars()
                .zip(0..=255) // iterate with y coordinate
                .filter(|(c, _)| *c == filled)
            {
                let id = points.len();
                points.push(Point { x, y });
                data[(x as usize, y as usize)] = Some(id);
            }
        }
        Image { data, points }
    }
}
