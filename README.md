---
author:
- |
    Polyomino covers\
    Maroš Grego
bibliography:
- 'user.bib'
title: User documentation
---

Introduction
============

An input to the following program consists of 1-bit image in the form of
a block of text (where one specific character, `x` by default,
represents a filled pixel and the rest of the characters represent an
unfilled pixel) and a list of 1-bit shapes - polyomino tiles. The
program, then (according to the user's choice) finds one or all possible
covers of the image by the polyomino tiles.

  -------------------------------------- ---------------------------------------
   ![image](pentomino.png){width="8cm"}   ![image](polytiling.png){width="5cm"}
         Example: pentomino tiles             One cover by pentomino tiles
  -------------------------------------- ---------------------------------------

Building and running
====================

This program is written in Rust and can be compiled using the standard
Rust toolchain, with Cargo as the package manager. The recommended
method of installing the Rust toolchain is Rustup [@rustup].

An optimised version of the code can be compiled with
`cargo build –release`. The compiled binary will then be located at
`target/release/polyomino-solver`.

Alternatively, the program can be compiled and run with
`cargo run –release`, with command line arguments passed after the `–`
argumnent, as in `cargo run –release – -r -b tiles/domino`.

The program consists of a library (at `src/lib.rs`), where all the
application logic is located and a binary (at `src/main.rs`), which is
just a wrapper around the library passing command line arguments to it.
The library has no external dependencies. The binary has one external
dependency, `structopt` crate for parsing the command line arguments
(unfortunately, in the present state dependencies for binary only can't
be specified in the Cargo config).

Input
=====

We can run the program as\
`polyomino-solver [-AOr] [-b blockfile] [-i inputfile] [-w wchar`\]\
The program reads text from the input and interprets it as 1-bit image
(every line of the text represents a line of pixels in the image, the
character `wchar` represents a white or filled pixel and any other
character represents a black or empty pixel; every line is completed
with empty pixels to the length of the largest line). The maximum
allowed width and height of the image is 255 pixels.

For the given image, the program finds all the covers by the polyomino
blocks defined in `blockfile`. The program perceives some isomorphic
covers (with respect to rotation or reflection) as separate covers.

The input needs to be a valid UTF-8 text. The program runs through
separate characters (opposed to the *grapheme clusters*, usually
perceived as separate letters), so it is recommended to only use the
characters that are displayed separately.

Command line arguments
======================

  ---------------- ------------------------------------------------------------------------------------------------
  `-A`             Prints all solutions (by default, only one solution and the number of solutions are printed)
  `-O`             Prints the first solution that is found and ends (by default all are found)
  `-r`             Enables repeated use of the tiles (by default, a block from every class can only be used once)
  `-i inputfile`   Loads the input from `inputfile` (defaults to the standard input)
  `-w wchar`       Interprets the character `wchar` as a filled pixel (defaults to `x`).
  `-b blockfile`   Loads the file with the polyomino tiles that can be used (defaults to blocks of pentomino)
  ---------------- ------------------------------------------------------------------------------------------------

In the `blockfile`, every lines represents one rendition of some
polyomino tile. The first character of the line identifies the class of
the tile. Every tile consists of at least one pixel, implicitly located
at the coordinates `[0,0]`. If the tile consists of more than one pixel,
the identifier is followed by tuples of their x and y coordinates,
separated by whitespace (max. 255). If the repetition is not allowed (by
the `-r` flag), there can be at most one tile from every class used.
This way, for example, rotations and reflections of a tile can be
practically considered as the same tile. E.g. the pentomino tile **P**
can be in with all of its isomorphisms denoted as:\
` P 1 0 0 1 1 1 0 2 P 1 0 2 0 1 1 2 1 P 1 0 0 1 1 1 2 1 P -1 1 0 1 -1 2 0 2 P 1 0 0 1 1 1 1 2 P 1 0 2 0 0 1 1 1 P 1 0 -1 1 0 1 1 1 P 0 1 1 1 0 2 1 2 `
Tiles belonging to the same class of tiles can be arbitrary, though.

Examples
========

` $ polyomino-solver `

    xxxxxxxx
    xxxxxxxx
    xxxxxxxx
    xxx..xxx
    xxx..xxx
    xxxxxxxx
    xxxxxxxx 
    xxxxxxxx

Output:

    +-+-+-+-+-+-+-+-+
    |U U|X|N N N|L L|
    + +-+ +-+-+ +-+ +
    |U|X X X|F|N N|L|
    + +-+ +-+ +-+-+ +
    |U U|X|F F F|T|L|
    +-+-+-+-+-+ + + +
    |I|Z Z|   |F|T|L|
    + +-+ +   +-+ +-+
    |I|Y|Z|   |T T T|
    + + + +-+-+-+-+-+
    |I|Y|Z Z|W|V V V|
    + + +-+-+ +-+-+ +
    |I|Y Y|W W|P P|V|
    + + +-+ +-+   + +
    |I|Y|W W|P P P|V|
    +-+-+-+-+-+-+-+-+

    520 solutions found in 330.248ms

` $ polyomino-solver -Or -w f`

    fffff
    .fffff
    ..fffff
    ...fffff
    ..fffff

Output:

    +-+-+-+-+-+
    |L L L L|N|
    +-+-+-+ + +-+
      |N N|L|N N|
      +-+ +-+-+ +-+
        |N N N|N|F|
        +-+-+-+ + +-+
          |P P|N|F F|
        +-+   +-+ +-+
        |P P P|F F|
        +-+-+-+-+-+

    1 solution found in 1.044ms

Testing
=======

Various tests can be run using `cargo test –release`, testing all the
options. Currently, the program passes every test.
