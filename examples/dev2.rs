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

    time::sleep(Duration::from_secs(20)).await;

    Ok(())
}
