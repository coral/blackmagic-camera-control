use blackmagic_camera_control::command::{Command, Display, Video};
fn main() {
    let m = Command::Display(Display::Brightness(2.0));

    //Command::Video(Video::Iso(200));
    dbg!(m);
}
