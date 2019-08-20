use std::collections::LinkedList;
use std::sync::Arc;
use std::thread;

use crate::heads::HeadList;
use crate::image::Image;
use crate::matrix::Matrix;
use crate::tiles::Tiles;

const EXPECTED_ROWS: usize = 3600;
const REMOVED_ROWS_CAP: usize = 256;
const REMOVED_COLS_CAP: usize = 32;
const SOLUTINON_CAP: usize = 32;
const MAX_PARALLEL_DEPTH: u16 = 2;

type Solution = Vec<usize>;

#[derive(Clone)]
pub struct Solver {
    matrix: Arc<Matrix<bool>>,
    pointcount: usize,
    maxima: Arc<Vec<u8>>,
    max_possible: usize,
    remaining: usize,
    cover_sizes: Arc<Vec<u8>>,
    cols: HeadList,
    rows: HeadList,
    solution: Solution,
    allow_repeat: bool,
}

fn build_matrix(image: &Image, tiles: &Tiles) -> (Matrix<bool>, Vec<u8>, Vec<u8>) {
    let width = image.points.len() + tiles.lookup.len();
    let mut buffer = Vec::with_capacity(32);
    let mut maxima = vec![0; tiles.lookup.len()];
    let mut matrix = Matrix::new(width, EXPECTED_ROWS);
    let mut cover_sizes = Vec::with_capacity(EXPECTED_ROWS);
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

            let new_row = matrix.add_row();
            for covered_point in buffer.iter() {
                matrix[(new_row, *covered_point)] = true;
            }

            let tilesize = tile.points.len() as u8 + 1;
            if tilesize > maxima[tile.kind] {
                maxima[tile.kind] = tilesize;
            }

            matrix[(new_row, image.points.len() + tile.kind)] = true;
            cover_sizes.push(tilesize);
            buffer.clear()
        }
    }

    (matrix, maxima, cover_sizes)
}

impl Solver {
    pub fn new(image: &Image, tiles: &Tiles, allow_repeat: bool) -> Self {
        let (matrix, maxima, cover_sizes) = build_matrix(image, tiles);
        let max_possible = maxima.iter().map(|&i| i as usize).sum();
        let width = matrix.width();
        let height = matrix.height();
        Solver {
            matrix: Arc::new(matrix),
            pointcount: image.points.len(),
            maxima: Arc::new(maxima),
            max_possible,
            remaining: image.points.len(),
            cover_sizes: Arc::new(cover_sizes),
            rows: HeadList::new(height),
            cols: HeadList::new(width),
            solution: Vec::with_capacity(SOLUTINON_CAP),
            allow_repeat,
        }
    }

    pub fn solve_next(&mut self, depth: u16) -> LinkedList<Solution> {
        let mut solutions = LinkedList::new();
        let (min, mincol) = match self
            .cols
            .iter()
            .filter(|&y| {
                !(self.allow_repeat || self.max_possible > self.remaining) || y < self.pointcount
            })
            .map(|y| (self.rows.iter().filter(|&x| self.matrix[(x, y)]).count(), y))
            .min_by_key(|&(count, _)| count)
        {
            Some(m) => m,
            None => {
                solutions.push_front(self.solution.clone());
                return solutions;
            }
        };
        if min == 0 {
            return solutions;
        };

        let (tr, tc) = (self.rows.terminal, self.cols.terminal);

        if depth < MAX_PARALLEL_DEPTH {
            let mut threads = Vec::with_capacity(min - 1);
            let mut row = self.rows[tr].next;
            let mut counter = 1; // ensuring not spawning thread for last row
            while row != tr {
                if self.matrix[(row, mincol)] {
                    if counter < min {
                        let mut solver = self.clone();
                        threads.push(thread::spawn(move || solver.solve_without_row(depth, row)));
                        counter += 1;
                    } else {
                        solutions.append(&mut self.solve_without_row(depth, row));
                    }
                }
                row = self.rows[row].next;
            }

            threads
                .into_iter()
                .map(|t| t.join().expect("A thread panicked!"))
                .fold(solutions, |mut acc, mut s| { acc.append(&mut s); acc })
        } else {
            let mut removed_rows = Vec::with_capacity(REMOVED_ROWS_CAP);
            let mut removed_cols = Vec::with_capacity(REMOVED_COLS_CAP);

            // Had to fake C-style for loops like a pig
            // because borrow checker wouldn't allow mutating data being iterated over
            let mut row = self.rows[tr].next;
            while row != tr {
                if self.matrix[(row, mincol)] {
                    let prev_max = self.max_possible;
                    let prev_rem = self.remaining;

                    let mut col = self.cols[tc].next;
                    while col != tc {
                        if self.matrix[(row, col)] {
                            let mut r = self.rows[tr].next;
                            while r != tr {
                                if self.matrix[(r, col)] {
                                    self.rows.remove(r);
                                    removed_rows.push(r);
                                }
                                r = self.rows[r].next;
                            }
                            self.cols.remove(col);
                            removed_cols.push(col);
                            if col >= self.pointcount {
                                self.max_possible -= self.maxima[col - self.pointcount] as usize;
                            }
                        }
                        col = self.cols[col].next;
                    }
                    self.remaining -= self.cover_sizes[row] as usize;

                    if self.remaining <= self.max_possible {
                        self.solution.push(row);
                        solutions.append(&mut self.solve_next(depth + 1));
                        self.solution.pop();
                    }

                    for rr in removed_rows.iter().rev() {
                        self.rows.restore(*rr);
                    }
                    removed_rows.clear();
                    for rc in removed_cols.iter().rev() {
                        self.cols.restore(*rc);
                    }
                    removed_cols.clear();

                    self.max_possible = prev_max;
                    self.remaining = prev_rem;
                }
                row = self.rows[row].next;
            }

            solutions
        }
    }

    fn solve_without_row(&mut self, depth: u16, row: usize) -> LinkedList<Solution> {
        let (tr, tc) = (self.rows.terminal, self.cols.terminal);
        let mut col = self.cols[tc].next;
        while col != tc {
            if self.matrix[(row, col)] {
                let mut r = self.rows[tr].next;
                while r != tr {
                    if self.matrix[(r, col)] {
                        self.rows.remove(r);
                    }
                    r = self.rows[r].next;
                }
                self.cols.remove(col);
                if col >= self.pointcount {
                    self.max_possible -= self.maxima[col - self.pointcount] as usize;
                }
            }
            col = self.cols[col].next;
        }
        self.remaining -= self.cover_sizes[row] as usize;

        if self.remaining <= self.max_possible {
            self.solution.push(row);
            self.solve_next(depth + 1)
        } else {
            LinkedList::new()
        }
    }
}
