use std::slice::Chunks;

// sizes
pub const TILE_SIZE: usize = 8;
pub const TILES_IN_ROW: usize = 4;
pub const TILES_IN_COL: usize = 4;
pub const BLOCK_SIZE: usize = TILE_SIZE * TILES_IN_ROW;
pub const TILES_IN_BLOCK: usize = TILES_IN_ROW * TILES_IN_COL;

// colors
pub enum Rgb { RED, GREEN, BLUE }

impl Rgb {
    pub fn from(i: usize) -> Self {
        match i % 3 {
            0 => Rgb::RED,
            1 => Rgb::GREEN,
            _ => Rgb::BLUE,
        }
    }
}

pub type RgbTriple = (u8, u8, u8);

pub fn rgb_triple_from(chunks: &[u8]) -> RgbTriple {
    (
        *chunks.get(0).unwrap(),
        *chunks.get(1).unwrap(),
        *chunks.get(2).unwrap(),
    )
}

pub type SubpixelPalette = (u8, u8, u8, u8);

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct RgbPalette(pub RgbTriple, pub RgbTriple, pub RgbTriple, pub RgbTriple);

impl RgbPalette {
    pub fn map_subpixel(&self, rgb: Rgb) -> SubpixelPalette {
        match rgb {
            Rgb::RED    => (self.0 .0, self.1 .0, self.2 .0, self.3 .0),
            Rgb::GREEN  => (self.0 .1, self.1 .1, self.2 .1, self.3 .1),
            Rgb::BLUE   => (self.0 .2, self.1 .2, self.2 .2, self.3 .2),
        }
    }
}

pub const BASE_PALETTE: RgbPalette = RgbPalette(
    (0,0,0),
    (0x55,0x55,0x55),
    (0xaa,0xaa,0xaa),
    (0xff,0xff,0xff),
);

pub const HOVER_PALETTE: RgbPalette = RgbPalette(
    (0,0,0),
    (13,73,80),
    (46,138,106),
    (253,244,152),

//    (0,0,0),
//    (9, 92, 104),
//    (196, 125, 0),
//    (234, 240, 102),
);

pub const SELECT_PALETTE: RgbPalette = RgbPalette(
    (0,0,0),
    (243, 84, 57),
    (246, 141, 92),
    (244, 210, 122),
);
