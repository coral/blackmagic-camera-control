use blackmagic_camera_control::command::{Command, Video};
use blackmagic_camera_control::BluetoothCamera;
use std::error::Error;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut camera = BluetoothCamera::new("A:4BE2529F".to_string())
        .await
        .unwrap();
    dbg!(camera.connect(Duration::from_secs(10)).await);

    dbg!("Connected");

    camera
        .write_command(255, Command::Video(Video::Iso(3200)))
        .await
        .unwrap();

    dbg!("OK");

    time::sleep(Duration::from_secs(20)).await;

    //state[blackmagic_camera_control::data::Media::Codec];

    Ok(())
}
