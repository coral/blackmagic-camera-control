use blackmagic_camera_control::command::{Command, Video};
use blackmagic_camera_control::BluetoothCamera;
use std::error::Error;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    //Create a new camera with the device name
    let mut camera = BluetoothCamera::new("A:4BE2529F".to_string())
        .await
        .unwrap();

    //Connect with a set timeout
    camera.connect(Duration::from_secs(10)).await.unwrap();

    //Change the ISO to 320
    camera
        .write(255, Command::Video(Video::Iso(320)))
        .await
        .unwrap();

    dbg!("OK");

    time::sleep(Duration::from_secs(10)).await;

    Ok(())
}
