use std::error::Error;
use std::time::Duration;

fn main() -> Result<(), Box<dyn Error>> {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/dev/ttyACM0".to_string());

    // The baud rate is ignored by a USB CDC device, but the API requires one.
    let mut port = serialport::new(&path, 115_200)
        .timeout(Duration::from_millis(500))
        .open()?;

    port.write_all(b"n")?;

    let mut reply = [0u8; 1];
    port.read_exact(&mut reply)?;
    println!("Sent 'n', device echoed {:?}", reply[0] as char);
    Ok(())
}
