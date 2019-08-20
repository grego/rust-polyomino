use std::collections::LinkedList;
use std::ops::{Index, IndexMut};
use std::sync::Arc;
use std::thread;

use crate::image::Image;
use crate::tiles::Tiles;

const SOLUTINON_CAP: usize = 32;
const LINKAGE_CAP: usize = 10000;
const MAX_PARALLEL_DEPTH: u16 = 2;

#[derive(Clone)]
pub struct Node {
    left: usize,
    right: usize,
    up: usize,
    down: usize,
    kind: usize,
    extra: usize,
}

type Solution = Vec<usize>;

#[derive(Clone)]
pub struct Linkage {
    width: usize,
    pointcount: usize,
    data: Vec<Node>,
    solution: Solution,
    maxima: Arc<Vec<u8>>,
    max_possible: usize,
    remaining: usize,
    allow_repeat: bool,
}

impl Linkage {
    pub fn build(image: &Image, tiles: &Tiles, allow_repeat: bool) -> Self {
        let pointcount = image.points.len();
        let width = pointcount + tiles.lookup.len();
        let mut buffer = Vec::with_capacity(32);

        let mut maxima = vec![0; tiles.lookup.len()];
        let mut linkage = Linkage::with_capacity(width, pointcount, LINKAGE_CAP);
        for tile in tiles.data.iter() {
            'imageloop: for (ord, point) in image.points.iter().enumerate() {
                buffer.push(ord);
                for p in tile.points.iter() {
                    let x = point.x + p.x;
                    let y = point.y + p.y;
                    if x < 0 || y < 0 {
                        buffer.clear();
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

                buffer.push(pointcount + tile.kind);
                linkage.add_row(&buffer, tile.kind);

                let tilesize = tile.points.len() as u8 + 1;
                if tilesize > maxima[tile.kind] {
                    maxima[tile.kind] = tilesize;
                }
                buffer.clear()
            }
        }

        linkage.data.shrink_to_fit();
        linkage.remaining = pointcount;
        linkage.max_possible = maxima.iter().map(|&i| i as usize).sum();
        linkage.maxima = Arc::new(maxima);
        linkage.allow_repeat = allow_repeat;

        linkage
    }

    fn with_capacity(width: usize, pointcount: usize, capacity: usize) -> Self {
        let mut data = Vec::with_capacity(capacity);
        for i in 0..=width {
            data.push(Node {
                left: if i != 0 { i - 1 } else { width },
                right: if i != width { i + 1 } else { 0 },
                up: i,
                down: i,
                kind: std::usize::MAX,
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
        }
    }

    fn remove_from_row(&mut self, i: usize) {
        let previous = self[i].left;
        let next = self[i].right;
        self[previous].right = next;
        self[next].left = previous;
    }

    fn remove_from_col(&mut self, i: usize) {
        let previous = self[i].up;
        let next = self[i].down;
        self[previous].down = next;
        self[next].up = previous;
    }

    fn return_to_row(&mut self, i: usize) {
        let previous = self[i].left;
        let next = self[i].right;
        self[previous].right = i;
        self[next].left = i;
    }

    fn return_to_col(&mut self, i: usize) {
        let previous = self[i].up;
        let next = self[i].down;
        self[previous].down = i;
        self[next].up = i;
    }

    fn add_row(&mut self, row: &[usize], kind: usize) {
        let orig_index = self.data.len();
        let width = self.width;
        for (ord, &i) in row.iter().filter(|&&i| i < width).enumerate() {
            let index = self.data.len();
            self.data.push(Node {
                left: if ord != 0 { index - 1 } else { orig_index },
                right: orig_index,
                up: self[i].up,
                down: i,
                kind: if i < self.pointcount { i } else { std::usize::MAX },
                extra: i,
            });
            self.return_to_row(index);
            self.return_to_col(index);
            self[i].extra += 1;
        }
    }

    fn cover_col(&mut self, col: usize) {
        if col >= self.pointcount {
            self.max_possible -= self.maxima[col - self.pointcount] as usize;
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

    fn uncover_col(&mut self, col: usize) {
        if col >= self.pointcount {
            self.max_possible += self.maxima[col - self.pointcount] as usize;
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
        self[head].right == head
    }

    fn find_min(&self) -> (usize, usize) {
        let head = self.width;
        let mut i = self[head].right;
        let (mut min, mut mincol) = (std::usize::MAX, head);
        while i != head
            && ((!self.allow_repeat && self.remaining == self.max_possible) || i < self.pointcount)
        {
            if self[i].extra < min {
                min = self[i].extra;
                mincol = i;
            }
            i = self[i].right;
        }
        (min, mincol)
    }

    fn push_solution(&mut self, i: usize) {
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

    pub fn solve_next(&mut self, depth: u16) -> LinkedList<Solution> {
        let mut solutions = LinkedList::new();
        if self.is_empty() {
            solutions.push_front(self.solution.clone());
            return solutions;
        }

        let (min, mincol) = self.find_min();
        if min == 0 {
            return solutions;
        }
        self.cover_col(mincol);


        if depth < MAX_PARALLEL_DEPTH {
            let mut threads = Vec::with_capacity(min - 1);
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
                    linkage.solve_next(depth + 1)
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
            solutions.append(&mut self.solve_next(depth + 1));

            threads
                .into_iter()
                .map(|t| t.join().expect("A thread panicked!"))
                .fold(solutions, |mut acc, mut s| { acc.append(&mut s); acc })
                
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
                solutions.append(&mut self.solve_next(depth + 1));
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
}

impl Index<usize> for Linkage {
    type Output = Node;

    fn index(&self, index: usize) -> &Self::Output {
        self.data.index(index)
    }
}

impl IndexMut<usize> for Linkage {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.data.index_mut(index)
    }
}
