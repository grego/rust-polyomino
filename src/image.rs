use crate::matrix::Matrix;
use crate::tiles::Point;

pub struct Image {
    data: Matrix<Option<usize>>,
    points: Vec<Point>,
    width: usize,
}

impl Image {
    pub fn load(reader: impl std::io::BufRead, filled: char) -> Self {
        let mut points = Vec::new();
        let mut data = Matrix::new(255, 255);
        let mut width = 0;
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
            if line.len() > width {
                width = line.len()
            }
        }
        Image {
            data,
            points,
            width,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.data.height()
    }

    pub fn pointcount(&self) -> usize {
        self.points.len()
    }

    pub fn get_point(&self, id: usize) -> Option<&Point> {
        self.points.get(id)
    }

    pub fn get_point_id(&self, x: i16, y: i16) -> Option<usize> {
        if x < 0 || y < 0 {
            None
        } else {
            self.data.get(x as usize, y as usize).and_then(|&o| o)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &Point> + '_ {
        self.points.iter()
    }
}
