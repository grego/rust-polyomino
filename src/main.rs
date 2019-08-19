pub mod image;
pub mod heads;
pub mod matrix;
pub mod solver;
pub mod tiles;

use tiles::Tiles;
use image::Image;
use solver::{CoverMatrix, Solver};

use std::fs::File;
use std::io::{stdin, BufReader};

fn main() -> std::io::Result<()> {
    let f = File::open("tiles/pentomino")?;
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/chess")?;
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');
    let matrix = CoverMatrix::build(&image, &tiles);


    println!("{}", Solver::new(matrix).solve_next(0).len());

    Ok(())
}
