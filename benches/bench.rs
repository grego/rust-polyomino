#![feature(test)]

extern crate test;

use polyomino::tiles::Tiles;
use polyomino::image::Image;
use polyomino::linkage::Linkage;

use std::fs::File;
use std::io::BufReader;

use test::Bencher;

#[bench]
fn pentomino_20x3(b: &mut Bencher) {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/rect20x3").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    b.iter(|| Linkage::build(&image, &tiles, false).solve(true));
}

#[bench]
fn pentomino_square(b: &mut Bencher) {
    let f = File::open("tiles/pentomino").unwrap();
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let i = File::open("images/chess").unwrap();
    let i = BufReader::new(i);
    let image = Image::load(i, 'x');

    b.iter(|| Linkage::build(&image, &tiles, false).solve(true));
}
