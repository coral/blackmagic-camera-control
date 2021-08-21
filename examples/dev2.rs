use blackmagic_camera_control::BluetoothCamera;
use std::error::Error;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let camera = BluetoothCamera::new("A:4BE2529F".to_string())
        .await
        .unwrap();
    camera.connect(Duration::from_secs(10)).await;

    Ok(())
}
