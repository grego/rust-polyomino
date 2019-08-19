use std::collections::LinkedList;
use std::sync::Arc;

use crate::heads::HeadList;
use crate::image::Image;
use crate::matrix::Matrix;
use crate::tiles::Tiles;

const EXPECTED_ROWS: usize = 3600;
const REMOVED_ROWS_CAP: usize = 256;
const REMOVED_COLS_CAP: usize = 32;
const SOLUTINON_CAP: usize = 32;
const MAX_PARALLEL_DEPTH: u16 = 3;

type Solution = Vec<usize>;

pub struct CoverMatrix {
    pointcount: usize,
    data: Matrix<bool>,
}

#[derive(Clone)]
pub struct Solver {
    matrix: Arc<CoverMatrix>,
    cols: HeadList,
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
    pub fn new(matrix: CoverMatrix) -> Self {
        let width = matrix.data.width();
        let height = matrix.data.height();
        Solver {
            rows: HeadList::new(height),
            cols: HeadList::new(width),
            matrix: Arc::new(matrix),
            solution: Vec::with_capacity(SOLUTINON_CAP),
        }
    }
    
    pub fn solve_next(&mut self, depth: u16) -> LinkedList<Solution> {
        let mut solutions = LinkedList::new();
        let (min, mincol) = match self
            .cols
            .iter()
            .map(|y| (self.rows.iter().filter(|&x| self.matrix.data[(x, y)]).count(), y))
            .min_by_key(|&(count, _)| count) {
                Some(m) => m,
                None => {
                    solutions.push_front(self.solution.clone());
                    return solutions;
                }
            };
        if min == 0 { return solutions };

        let mut removed_rows = Vec::with_capacity(REMOVED_ROWS_CAP);
        let mut removed_cols = Vec::with_capacity(REMOVED_COLS_CAP);

        //println!("{}", depth);

        // Had to fake C-style for loops like a pig
        // because borrow checker wouldn't allow mutating data being iterated over
        let (tr, tc) = (self.rows.terminal, self.cols.terminal);
        let mut row = self.rows.data[tr].next;
        while row != tr {
            if self.matrix.data[(row, mincol)] {
                let mut col = self.cols.data[tc].next;
                while col != tc {
                    if self.matrix.data[(row, col)] {
                        let mut r = self.rows.data[tr].next;
                        while r != tr {
                            if self.matrix.data[(r, col)] {
                                self.rows.remove(r);
                                removed_rows.push(r);
                            }
                            r = self.rows.data[r].next;
                        }
                        self.cols.remove(col);
                        removed_cols.push(col);
                    }
                    col = self.cols.data[col].next;
                }
                self.solution.push(row);
                solutions.append(&mut self.solve_next(depth + 1));

                self.solution.pop();
                for rr in removed_rows.iter().rev {
                    self.rows.restore(*rr);
                }
                removed_rows.clear();
                for rc in removed_cols.iter().rev {
                    self.cols.restore(*rc);
                }
                removed_cols.clear();
            }
            row = self.rows.data[row].next;
        }


/*       for row in self.rows.iter().filter(|&x| self.matrix.data[(x, mincol)]) {
            for col in self.cols.iter().filter(|&y| self.matrix.data[(row, y)]) {
                for row in rows.iter().filter(|&x| self.matrix.data[(x, col)]) {
                    self.rows.remove(row);
                    removed_rows.push(row);
                }
                self.cols.remove(col);
                removed_cols.push(col);
            }
            self.rows.remove(row);
            removed_rows.push(row);
            self.solution.push(row);
            solutions.append(&mut self.solve_next(depth + 1));

            for rr in removed_rows.iter().rev() {
                self.rows.restore(*rr);
            }
            for rc in removed_cols.iter().rev() {
                self.cols.restore(*rc);
            }
        } */

        solutions
    }
}
