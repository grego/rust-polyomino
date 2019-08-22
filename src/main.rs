use polyomino::image::Image;
use polyomino::solver::Solver;
use polyomino::linkage::Linkage;
use polyomino::tiles::Tiles;

use std::fs::File;
use std::io::{stdin, BufReader};
use std::time::{Duration, Instant};

fn main() -> std::io::Result<()> {
    let f = File::open("tiles/pentomino")?;
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect5x4")?;
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let start = Instant::now();
    let solutions = Linkage::build(&image, &tiles, true).solve_next(0);
    let duration = start.elapsed();
    println!("{} solutions, found in: {:?}", solutions.len(), duration);

    Ok(())
}
