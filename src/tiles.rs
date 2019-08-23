use std::collections::HashMap;

pub struct Point {
    pub x: i16,
    pub y: i16,
}

pub struct Tile {
    pub kind: usize,
    pub points: Vec<Point>,
}

pub struct Tiles {
    kinds: Vec<char>,
    data: Vec<Tile>,
}

impl Tile {
    fn parse(s: &str, used: &mut HashMap<char, usize>, lookup: &mut Vec<char>) -> Option<Self> {
        let mut words = s.split_whitespace();
        let identifier = words.next()?.chars().next()?;
        let kind = *used.entry(identifier).or_insert_with(|| {
            let new_kind = lookup.len();
            lookup.push(identifier);
            new_kind
        });
        let points = words
            .filter_map(|w| w.parse().ok())
            .collect::<Vec<_>>()
            .chunks_exact(2)
            .take(254)
            .map(|c| Point { x: c[0], y: c[1] })
            .collect();
        Some(Tile { kind, points })
    }
}

impl Tiles {
    pub fn load(reader: impl std::io::BufRead) -> Self {
        let mut used = HashMap::new();
        let mut kinds = Vec::new();
        let data = reader
            .lines()
            .filter_map(Result::ok)
            .filter_map(|s| Tile::parse(&s, &mut used, &mut kinds))
            .collect();
        Tiles { kinds, data }
    }

    pub fn kinds_count(&self) -> usize {
        self.kinds.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Tile> + '_ {
        self.data.iter()
    }

    pub fn name(&self, kind: usize) -> char {
        self.kinds[kind]
    }
}
