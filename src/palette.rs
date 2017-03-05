use gdk_pixbuf::Pixbuf;

use constants::*;

pub fn change_palette(tile: &Pixbuf, from_pal: RgbPalette, to_pal: RgbPalette) -> Pixbuf {
    let mut pxs: Vec<u8> = Vec::new();
    unsafe {
        for chunk in tile.get_pixels().iter().as_slice().chunks(3) {
            let triple = rgb_triple_from(chunk);
            let (red, green, blue) = {
                if triple == from_pal.0 {
                    to_pal.0
                } else if triple == from_pal.1 {
                    to_pal.1
                } else if triple == from_pal.2 {
                    to_pal.2
                } else if triple == from_pal.3 {
                    to_pal.3
                } else {
                    triple //panic!("{:?} match not found: {:?} {:?} {:?}", i, triple, from_pal, to_pal);
                }
            };

            pxs.push(red);
            pxs.push(green);
            pxs.push(blue);
        }
    };
    Pixbuf::new_from_vec(pxs, tile.get_colorspace(), false, tile.get_bits_per_sample(), tile.get_width(), tile.get_height(), tile.get_rowstride())
}
