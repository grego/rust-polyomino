#![feature(test)]

use polyomino::image::Image;
use polyomino::solver::Solver;
use polyomino::tiles::Tiles;

use std::fs::File;
use std::io::{stdin, BufReader};
use std::time::{Duration, Instant};

#[test]
fn pentomino_chess() {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/chess").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let start = Instant::now();
    let solutions = Solver::new(&image, &tiles, false).solve_next(0);
    let duration = start.elapsed();

    println!("All solutions for pentomino chess found in: {:?}", duration);
    assert_eq!(solutions.len(), 520);
}
