# Interface with your Blackmagic camera over Bluetooth in Rust!

This library allows you to easily communicate with your Blackmagic camera over Bluetooth.

-   Implements the full camera spec for easy access to commands with static type checking `Command::Video(Video::Iso(640)))`
-   Uses [btleplug](https://github.com/deviceplug/btleplug) for Bluetooth to work across platforms.
-   Consumes `PROTOCOL.json` for code generation so it's easy to add more functions to the library

## Usage

You can test the library easy by opening `examples/control.rs`, replacing the _CAMERA_NAME_ const with your cameras bluetooth name and then running `cargo run --example control`

```rust
//Create a new camera with the device name
let mut camera = BluetoothCamera::new(CAMERA_NAME).await.unwrap();

//Connect with a set timeout
camera.connect(Duration::from_secs(10)).await.unwrap();

//Change the ISO to 320
camera.write(255, Operation::AssignValue, Command::Video(Video::Iso(640))).await.unwrap();
```

## How does it work?

The library consumes the [PROTOCOL.json](https://github.com/coral/blackmagic-camera-protocol) file which documents the camera protocol in a machine readable format. From there it generates the commands as rust enums during the build stage (see /build). This allows us to have statically typed addressing of camera features without manually writing the code, rather relying on the conversion from the camera protocol manual. The library takes care of packaging down the commands into the camera protocol.

## Contributing

Just open a PR LUL

## License

All under MIT
