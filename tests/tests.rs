use polyomino::tiles::Tiles;
use polyomino::image::Image;
use polyomino::linkage::Linkage;

use std::fs::File;
use std::io::BufReader;

#[test]
fn pentomino_chess() {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/chess").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, false).solve(true);
    assert_eq!(solutions.len(), 520);
}

#[test]
fn pentomino_rect3x20() {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect20x3").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, false).solve(true);
    assert_eq!(solutions.len(), 8);
}

#[test]
fn pentomino_rect4x15() {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect15x4").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, false).solve(true);
    assert_eq!(solutions.len(), 1472);
}

#[test]
fn pentomino_rect4x5() {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect5x4").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, false).solve(true);
    assert_eq!(solutions.len(), 200);
}

#[test]
fn tromino_simple() {
    let f = File::open("tiles/tromino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect3x1").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, false).solve(true);
    assert_eq!(solutions.len(), 1);
}

#[test]
fn domino_repeat() {
    let f = File::open("tiles/domino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect3x2").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, true).solve(true);
    assert_eq!(solutions.len(), 3);
}

#[test]
fn find_one() {
    let f = File::open("tiles/pentomino_square").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect8x8").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    let solutions = Linkage::build(&image, &tiles, false).solve(false);
    assert_eq!(solutions.len(), 1);
}
