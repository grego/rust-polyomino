use std::collections::LinkedList;
use std::sync::Arc;

use crate::heads::HeadList;
use crate::image::Image;
use crate::matrix::Matrix;
use crate::tiles::Tiles;

const EXPECTED_ROWS: usize = 3600;
const REMOVED_ROWS_CAP: usize = 256;
const REMOVED_COLS_CAP: usize = 32;
const MAX_PARALLEL_DEPTH: u16 = 3;

type Solution = Vec<usize>;

pub struct CoverMatrix {
    pointcount: usize,
    data: Matrix<bool>,
}

#[derive(Clone)]
pub struct Solver {
    matrix: Arc<CoverMatrix>,
    columns: HeadList,
    rows: HeadList,
    solution: Solution,
}

impl CoverMatrix {
    pub fn build(image: &Image, tiles: &Tiles) -> Self {
        let pointcount = image.points.len();
        let width = pointcount + tiles.lookup.len();

        let mut buffer = Vec::with_capacity(32);
        let mut data = Matrix::new(width, EXPECTED_ROWS);
        for tile in tiles.data.iter() {
            'imageloop: for (ord, point) in image.points.iter().enumerate() {
                buffer.push(ord);
                for p in tile.points.iter() {
                    let x = point.x + p.x;
                    let y = point.y + p.y;
                    if x < 0 || y < 0 {
                        continue 'imageloop;
                    }

                    if let Some(nextpoint) = image.data.get(x as usize, y as usize).and_then(|&o| o)
                    // Option::flatten is currently nigtly-only
                    {
                        buffer.push(nextpoint);
                    } else {
                        buffer.clear();
                        continue 'imageloop;
                    }
                }

                let new_row = data.add_row();
                for covered_point in buffer.iter() {
                    data[(new_row, *covered_point)] = true;
                }
                data[(new_row, pointcount + tile.kind)] = true;
                buffer.clear();
            }
        }

        CoverMatrix { pointcount, data }
    }
}

impl Solver {
    fn solve_next(&mut self, depth: u16) -> LinkedList<Solution> {
        let solutions = LinkedList::new();
        let (min, mincol) = match self
            .columns
            .iter()
            .map(|y| (self.rows.iter().filter(|&x| self.matrix.data[(x, y)]).count(), y))
            .min_by_key(|(_, count)| count) {
                Some(m) => m,
                None => {
                    solutions.push_front(self.solution.clone());
                    return solutions;
                }
            };
        if min == 0 { return solutions };

        let removed_rows = Vec::with_capacity(REMOVED_ROWS_CAP);
        let removed_cols = Vec::with_capacity(REMOVED_COLS_CAP);
        for row in self.rows.iter().filter(|&x| self.matrix.data[(x, mincol)]) {
            for column in self.columns.iter().filter(|&y| self.matrix.data[(row, y)]) {
                for row in self.rows.iter().filter(|&x| self.matrix.data[(x, column)]) {
                    self.rows.remove(row);
                    removed_rows.push(row);
                }
                self.columns.remove(column);
                removed_cols.push(column);
            }
            self.rows.remove(row);
            removed_rows.push(row);
            self.solution.push(row);
            solutions.append(&mut self.solve_next(depth + 1));

            for rr in removed_rows.iter().rev() {
                self.rows.restore(*rr);
            }
            for rc in removed_cols.iter().rev() {
                self.columns.restore(*rc);
            }
        }

        solutions
    }
}
