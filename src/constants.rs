extern crate gdk_sys;

pub const TILE_SIZE: usize = 8;
pub const TILES_IN_ROW: usize = 4;
pub const TILES_IN_COL: usize = 4;
pub const BLOCK_SIZE: usize = TILE_SIZE * TILES_IN_ROW;
pub const TILES_IN_BLOCK: usize = TILES_IN_ROW * TILES_IN_COL;

pub fn drawing_area_mask_bits() -> i32 {
    (gdk_sys::GDK_POINTER_MOTION_MASK
    | gdk_sys::GDK_BUTTON_PRESS_MASK
    | gdk_sys::GDK_BUTTON1_MOTION_MASK
    | gdk_sys::GDK_ENTER_NOTIFY_MASK
    | gdk_sys::GDK_LEAVE_NOTIFY_MASK).bits() as i32
}
