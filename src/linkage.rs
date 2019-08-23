use std::collections::LinkedList;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use std::thread;

use crate::image::Image;
use crate::matrix::Matrix;
use crate::tiles::Tiles;

const SOLUTINON_CAP: usize = 32;
const LINKAGE_CAP: usize = 10000;
const MAX_PARALLEL_DEPTH: u16 = 2;

#[derive(Clone)]
pub struct Node {
    left: u32,
    right: u32,
    up: u32,
    down: u32,
    extra: u32, // #nodes for column headers, pointer to respective column header otherwise
}

type Solution = Vec<u32>;

// a 2D linkage of nodes
// first (width) nodes are column headers, followed by a main node pointing to
// the first and last column header, then the actual nodes
#[derive(Clone)]
pub struct Linkage {
    width: u32,      // #columns
    pointcount: u32, // #columns representing points in the image
    data: Vec<Node>,
    solution: Solution,   // a solution build so far
    maxima: Arc<Vec<u8>>, // max #points in each tile class
    max_possible: u32,    // size of the largest image that can be built with remaining tiles
    remaining: u32,       // #remaining tiles
    allow_repeat: bool,
    unused: bool, // are some of the tiles unused?
}

struct LinkageIterRow<'a> {
    first: u32,
    current: u32,
    linkage: &'a Linkage,
}

impl Linkage {
    pub fn build(image: &Image, tiles: &Tiles, allow_repeat: bool) -> Self {
        let pointcount = image.pointcount();
        let width = pointcount + tiles.kinds_count();
        let mut buffer = Vec::with_capacity(32);

        let mut maxima = vec![0; tiles.kinds_count()];
        let mut linkage = Linkage::with_capacity(width as u32, pointcount as u32, LINKAGE_CAP);
        for tile in tiles.iter() {
            'imageloop: for (ord, point) in image.iter().enumerate() {
                buffer.push(ord);
                for p in tile.points.iter() {
                    if let Some(nextpoint) = image.get_point_id(point.x + p.x, point.y + p.y) {
                        buffer.push(nextpoint);
                    } else {
                        buffer.clear();
                        continue 'imageloop;
                    }
                }

                buffer.push(pointcount + tile.kind);
                linkage.add_row(&buffer);

                let tilesize = tile.points.len() as u8 + 1;
                if tilesize > maxima[tile.kind] {
                    maxima[tile.kind] = tilesize;
                }
                buffer.clear()
            }
        }

        let mut i = pointcount as u32;
        while i != width as u32 {
            if linkage[i].down == i {
                linkage.unused = true;
                break;
            }
            i = linkage[i].right;
        }

        linkage.data.shrink_to_fit();
        linkage.remaining = pointcount as u32;
        linkage.max_possible = maxima.iter().map(|&i| i as u32).sum();
        linkage.maxima = Arc::new(maxima);
        linkage.allow_repeat = allow_repeat;

        linkage
    }

    fn with_capacity(width: u32, pointcount: u32, capacity: usize) -> Self {
        let mut data = Vec::with_capacity(capacity);
        for i in 0..=width {
            data.push(Node {
                left: if i != 0 { i - 1 } else { width },
                right: if i != width { i + 1 } else { 0 },
                up: i,
                down: i,
                extra: 0,
            });
        }
        Linkage {
            width,
            pointcount,
            data,
            solution: Vec::with_capacity(SOLUTINON_CAP),
            maxima: Arc::new(Vec::with_capacity(0)),
            remaining: 0,
            max_possible: 0,
            allow_repeat: false,
            unused: false,
        }
    }

    fn remove_from_row(&mut self, i: u32) {
        let previous = self[i].left;
        let next = self[i].right;
        self[previous].right = next;
        self[next].left = previous;
    }

    fn remove_from_col(&mut self, i: u32) {
        let previous = self[i].up;
        let next = self[i].down;
        self[previous].down = next;
        self[next].up = previous;
    }

    fn return_to_row(&mut self, i: u32) {
        let previous = self[i].left;
        let next = self[i].right;
        self[previous].right = i;
        self[next].left = i;
    }

    fn return_to_col(&mut self, i: u32) {
        let previous = self[i].up;
        let next = self[i].down;
        self[previous].down = i;
        self[next].up = i;
    }

    fn add_row(&mut self, row: &[usize]) {
        let orig_index = self.data.len() as u32;
        let width = self.width;
        for (ord, i) in row
            .iter()
            .map(|&i| i as u32)
            .filter(|&i| i < width)
            .enumerate()
        {
            let index = self.data.len() as u32;
            self.data.push(Node {
                left: if ord != 0 { index - 1 } else { orig_index },
                right: orig_index,
                up: self[i].up,
                down: i,
                extra: i,
            });
            self.return_to_row(index);
            self.return_to_col(index);
            self[i].extra += 1;
        }
    }

    // remove the column from the list of column headers
    // and every element on every row of the column from their respective columns
    fn cover_col(&mut self, col: u32) {
        if col >= self.pointcount {
            if self.allow_repeat {
                return;
            }
            self.max_possible -= self.maxima[(col - self.pointcount) as usize] as u32;
        }

        self.remove_from_row(col);
        let mut i = self[col].down;
        while i != col {
            let mut j = self[i].right;
            while j != i {
                self.remove_from_col(j);
                let head = self[j].extra;
                self[head].extra -= 1;
                j = self[j].right;
            }
            i = self[i].down;
        }
    }

    // revert cover_col
    fn uncover_col(&mut self, col: u32) {
        if col >= self.pointcount {
            if self.allow_repeat {
                return;
            }
            self.max_possible += self.maxima[(col - self.pointcount) as usize] as u32;
        }

        let mut i = self[col].up;
        while i != col {
            let mut j = self[i].left;
            while j != i {
                self.return_to_col(j);
                let head = self[j].extra;
                self[head].extra += 1;
                j = self[j].left;
            }
            i = self[i].up;
        }
        self.return_to_row(col);
    }

    fn is_empty(&self) -> bool {
        let head = self.width;
        let i = self[head].right;
        !(i != head
            && ((!self.allow_repeat && !self.unused && self.remaining == self.max_possible)
                || i < self.pointcount))
    }

    fn find_min(&self) -> (u32, u32) {
        let head = self.width;
        let mut i = self[head].right;
        let (mut min, mut mincol) = (std::u32::MAX, 0);
        while i != head
            && ((!self.allow_repeat && !self.unused && self.remaining == self.max_possible)
                || i < self.pointcount)
        {
            if self[i].extra < min {
                min = self[i].extra;
                mincol = i;
            }
            i = self[i].right;
        }
        (min, mincol)
    }

    fn push_solution(&mut self, i: u32) {
        self.solution.push(i);
        let mut j = self[i].right;
        while j != i {
            self.remaining -= 1;
            j = self[j].right;
        }
    }

    fn pop_solution(&mut self) {
        if let Some(i) = self.solution.pop() {
            let mut j = self[i].right;
            while j != i {
                self.remaining += 1;
                j = self[j].right;
            }
        }
    }

    fn solve_next(&mut self, depth: u16, find_all: bool) -> LinkedList<Solution> {
        let mut solutions = LinkedList::new();
        if self.is_empty() {
            solutions.push_front(self.solution.clone());
            return solutions;
        }

        let (min, mincol) = self.find_min();
        if min == 0 || (!self.allow_repeat && self.max_possible < self.remaining) {
            return solutions;
        }
        self.cover_col(mincol);

        if depth < MAX_PARALLEL_DEPTH && find_all {
            let mut threads = Vec::with_capacity(min as usize - 1);
            let mut i = self[mincol].down;
            let second_last = self[mincol].up;
            while i != second_last {
                let mut linkage = self.clone();
                threads.push(thread::spawn(move || {
                    let mut j = linkage[i].right;
                    while j != i {
                        let col = linkage[j].extra;
                        linkage.cover_col(col);
                        j = linkage[j].right;
                    }
                    linkage.push_solution(i);
                    linkage.solve_next(depth + 1, find_all)
                }));
                i = self[i].down;
            }

            let mut j = self[i].right;
            while j != i {
                let col = self[j].extra;
                self.cover_col(col);
                j = self[j].right;
            }
            self.push_solution(i);
            solutions.append(&mut self.solve_next(depth + 1, find_all));

            threads
                .into_iter()
                .map(|t| t.join().expect("A thread panicked!"))
                .fold(solutions, |mut acc, mut s| {
                    acc.append(&mut s);
                    acc
                })
        } else {
            let mut i = self[mincol].down;
            while i != mincol {
                let mut j = self[i].right;
                while j != i {
                    let col = self[j].extra;
                    self.cover_col(col);
                    j = self[j].right;
                }

                self.push_solution(i);
                solutions.append(&mut self.solve_next(depth + 1, find_all));
                if !solutions.is_empty() && !find_all {
                    return solutions;
                }
                self.pop_solution();

                j = self[i].left;
                while j != i {
                    let col = self[j].extra;
                    self.uncover_col(col);
                    j = self[j].left;
                }

                i = self[i].down;
            }
            self.uncover_col(mincol);

            solutions
        }
    }

    fn iter_row<'a>(&'a self, first: u32) -> LinkageIterRow<'a> {
        LinkageIterRow {
            first,
            current: first,
            linkage: &self,
        }
    }

    pub fn solve(&mut self, find_all: bool) -> LinkedList<Solution> {
        self.solve_next(0, find_all)
    }

    pub fn show_solution(&self, solution: &Solution, image: &Image, tiles: &Tiles) -> String {
        let mut canvas = Matrix::new_fixed(2 * image.width() + 1, 2 * image.height() + 1, ' ');
        for &i in solution {
            let mut r = i;
            while self[r].extra < self.pointcount {
                r = self[r].right
            }
            let name = tiles.name((self[r].extra - self.pointcount) as usize);

            let mut row_iter = self
                .iter_row(r)
                .filter_map(|j| image.get_point(self[j].extra as usize))
                .map(|p| (2 * p.x as usize + 1, 2 * p.y as usize + 1));
            for (x, y) in &mut row_iter {
                canvas[(x, y)] = name;
                for &(x, y) in neighbours(x, y).into_iter() {
                    canvas[(x, y)] = if canvas[(x, y)] == '/' { ' ' } else { '/' };
                }
            }
            for (x, y) in row_iter {
                for &(x, y) in neighbours(x, y).into_iter() {
                    if canvas[(x, y)] == '/' {
                        if x % 2 == 1 {
                            canvas[(x, y)] = '|';
                            canvas[(x - 1, y)] = '+';
                            canvas[(x + 1, y)] = '+';
                        } else {
                            canvas[(x, y)] = '-';
                            canvas[(x, y - 1)] = '+';
                            canvas[(x, y + 1)] = '+';
                        }
                    }
                }
            }
        }

        canvas.iter_rows().fold("".to_string(), |acc, slice| {
            format!("{}{}\n", acc, slice.iter().collect::<String>())
        })
    }
}

fn neighbours(x: usize, y: usize) -> [(usize, usize); 4] {
    [(x - 1, y), (x, y - 1), (x + 1, y), (x, y + 1)]
}

impl Index<u32> for Linkage {
    type Output = Node;

    fn index(&self, index: u32) -> &Self::Output {
        self.data.index(index as usize)
    }
}

impl IndexMut<u32> for Linkage {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        self.data.index_mut(index as usize)
    }
}

impl<'a> Iterator for LinkageIterRow<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = self.linkage[self.current].right;
        if self.current != self.first {
            Some(self.current)
        } else {
            None
        }
    }
}
