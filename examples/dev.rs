use blackmagic_camera_control::{self, data::Category, data::Data, data::Video, Message};
use btleplug::api::{Central, Manager as _, Peripheral as _};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

async fn find_camera(central: &Adapter) -> Option<Peripheral> {
    for p in central.peripherals().await.unwrap() {
        if p.properties()
            .await
            .unwrap()
            .unwrap()
            .local_name
            .iter()
            .any(|name| name.contains("A:4BE2529F"))
        {
            return Some(p);
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    //let ourchar = Uuid::parse_str("B864E140-76A0-416A-BF30-5876504537D9")?;
    let ourchar = Uuid::parse_str("5DD3465F-1AEE-4299-8493-D2ECA2F8E1BB")?;

    pretty_env_logger::init();

    let manager = Manager::new().await?;

    let central = manager
        .adapters()
        .await
        .expect("Unable to fetch adapter list.")
        .into_iter()
        .nth(0)
        .expect("Unable to find adapters.");

    central.start_scan().await?;

    time::sleep(Duration::from_secs(2)).await;

    let camera = find_camera(&central).await.expect("No camera found");

    let camera_properties = camera.properties().await.unwrap().unwrap();
    println!(
        "{:?} {:?} {:?}",
        camera_properties.local_name,
        camera_properties.manufacturer_data,
        camera_properties.address
    );

    camera.connect().await?;

    time::sleep(Duration::from_millis(200)).await;

    let chars = camera.characteristics();

    for c in chars.iter() {
        dbg!(&c);
    }

    let char = chars.iter().find(|c| c.uuid == ourchar).expect("mskekg");

    loop {
        let vm = vec![160, 320, 640, 800, 1000, 1600, 3200];
        for i in vm {
            let message = blackmagic_camera_control::Message::create_message(
                255,
                Category::Video(Video::GainISOValue),
                blackmagic_camera_control::data::Operation::AssignValue,
                Data::Signed32(vec![i]),
            );

            camera
                .write(&char, &message, btleplug::api::WriteType::WithResponse)
                .await?;

            time::sleep(Duration::from_millis(50)).await;
        }
    }

    // camera.subscribe(&char).await?;
    // let mut notification_stream = camera.notifications().await?;
    // // Process while the BLE connection is not broken or stopped.
    // while let Some(data) = notification_stream.next().await {
    //     // println!("Received data from  [{:?}]: {:?}", data.uuid, data.value);
    //     match bmc_camera_control::Message::parse_message(data.value) {
    //         Ok(v) => {
    //             println!("{:?}", &v);
    //         }
    //         Err(e) => {}
    //     };
    // }

    Ok(())
}
