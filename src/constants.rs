// sizes
pub const TILE_SIZE: usize = 8;
pub const TILES_IN_ROW: usize = 4;
pub const BLOCK_SIZE: usize = TILE_SIZE * TILES_IN_ROW;

pub type RgbTriple = (u8, u8, u8);

pub fn rgb_triple_from(chunks: &[u8]) -> RgbTriple {
    (
        *chunks.get(0).unwrap(),
        *chunks.get(1).unwrap(),
        *chunks.get(2).unwrap(),
    )
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct RgbPalette(pub RgbTriple, pub RgbTriple, pub RgbTriple, pub RgbTriple);

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
