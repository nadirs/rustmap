extern crate gdk_sys;

// make moving clones into closures more convenient
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

macro_rules! drawing_area_mask_bits {
    () => (
        (gdk_sys::GDK_POINTER_MOTION_MASK
         | gdk_sys::GDK_BUTTON_PRESS_MASK
         | gdk_sys::GDK_BUTTON1_MOTION_MASK
         | gdk_sys::GDK_ENTER_NOTIFY_MASK
         | gdk_sys::GDK_LEAVE_NOTIFY_MASK).bits() as i32
    )
}
