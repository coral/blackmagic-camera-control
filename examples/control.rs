use blackmagic_camera_control::command::{Command, Metadata, Video};
use blackmagic_camera_control::BluetoothCamera;
use std::error::Error;
use std::time::Duration;
use tokio::time;

const CAMERA_NAME: &'static str = "A:4BE2529F";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //Create a new camera with the device name
    let mut camera = BluetoothCamera::new(CAMERA_NAME).await.unwrap();

    //Connect with a set timeout
    camera.connect(Duration::from_secs(10)).await.unwrap();

    //Change the ISO to 320
    camera
        .write(255, Command::Video(Video::Iso(640)))
        .await
        .unwrap();

    //Subscribe to updates from the camera;
    let mut updates = camera.updates().await;

    tokio::spawn(async move {
        loop {
            let update = updates.recv().await;
            match update {
                Ok(v) => {
                    println!("{:?}", v);
                }
                Err(_) => {}
            }
        }
    });

    time::sleep(Duration::from_secs(5)).await;

    // Get a specific piece of info from the cached properties
    let info = camera
        .get(Command::Metadata(Metadata::LensDistance("".to_string())))
        .await;
    dbg!(info);

    Ok(())
}
