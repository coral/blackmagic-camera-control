use blackmagic_camera_control::command::{Command, Display};
fn main() {
    let m = Command::Display(Display::Brightness(2.0));

    dbg!(m);
}
