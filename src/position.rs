use constants::*;

pub trait Positionable {
    fn get_event_pos(&self, pos: (f64, f64)) -> (u8, u8) {
        get_event_pos(pos)
    }
}

pub fn get_event_pos(pos: (f64, f64)) -> (u8, u8) {
    let (x, y) = pos;
    ((x / BLOCK_SIZE as f64) as u8, (y / BLOCK_SIZE as f64) as u8)
}
