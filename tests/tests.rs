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

    let solutions = Linkage::build(&image, &tiles, false).find_solutions();
    assert_eq!(solutions.len(), 520);
}
