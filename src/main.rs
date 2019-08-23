use polyomino::image::Image;
use polyomino::linkage::Linkage;
use polyomino::tiles::Tiles;

use std::fs::File;
use std::io::{stdin, BufReader};
use std::path::PathBuf;
use std::time::Instant;

use structopt::StructOpt;

#[derive(StructOpt)]
struct Opt {
    /// Print all solutions
    #[structopt(short = "A", long = "all")]
    print_all: bool,
    /// Finish after finding the first solution
    #[structopt(short = "O", long = "one")]
    find_one: bool,
    /// Allow repetition of blocks
    #[structopt(short = "r", long = "allow-repeat")]
    allow_repeat: bool,
    /// Read input from <inputfile>, defaults to standard input
    #[structopt(short = "i", parse(from_os_str))]
    inputfile: Option<PathBuf>,
    /// Interpret <wchar> as "filled" pixel in the input
    #[structopt(short = "w", default_value = "x")]
    wchar: char,
    /// Load blocks from <blockfile>
    #[structopt(short = "b", default_value = "tiles/pentomino", parse(from_os_str))]
    blockfile: PathBuf,
}

fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    let f = File::open(opt.blockfile)?;
    let f = BufReader::new(f);
    let tiles = Tiles::load(f);

    let image = match opt.inputfile {
        Some(f) => {
            let i = BufReader::new(File::open(f)?);
            Image::load(i, opt.wchar)
        }
        None => Image::load(BufReader::new(stdin().lock()), opt.wchar),
    };

    let start = Instant::now();
    let mut linkage = Linkage::build(&image, &tiles, opt.allow_repeat);
    let solutions = linkage.solve(!opt.find_one);
    let duration = start.elapsed();
    let len = solutions.len();
    for s in solutions.iter().take(if opt.print_all { len } else { 1 }) {
        println!("{}", linkage.show_solution(s, &image, &tiles))
    }
    println!(
        "{} solution{}, found in: {:?}",
        len,
        if len > 1 { "s" } else { "" },
        duration
    );
    if solutions.len() == 0 {
        println!("Allowing repetition (-r flag) could help find some.");
    }

    Ok(())
}
