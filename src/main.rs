use anyhow::Result;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};

use std::io::Write;
use std::net::TcpStream;

const CLOCK_FREQUENCY: u32 = 180_000;
const START: u8 = 0b0000_0001;
const CONFIG: u8 = 0b0010_0000;

fn create_spi() -> Result<Spidev> {
    let mut spi = Spidev::open("/dev/spidev0.0")?;
    let options = SpidevOptions::new()
        .bits_per_word(8)
        .max_speed_hz(CLOCK_FREQUENCY)
        .mode(SpiModeFlags::SPI_MODE_1)
        .build();
    spi.configure(&options)?;
    Ok(spi)
}

fn create_tcp_stream() -> Result<TcpStream> {
    let stream = TcpStream::connect("10.42.0.1:8080")?;
    Ok(stream)
}

fn transfer_data(spi: &mut Spidev, stream: &mut TcpStream) -> Result<()> {
    let tx_buf = [START, CONFIG, 0];
    let mut rx_buf = [0; 3];
    let mut val_as_bytes = [0; 4];
    let mut val: u32;
    let mask = 0xFFF;
    let vol_ref = 5f32;
    loop {
        if *rx_buf.last().unwrap() != 0 {
            rx_buf.fill_with(|| 0);
        }

        let mut transfer = SpidevTransfer::read_write(&tx_buf, &mut rx_buf);
        spi.transfer(&mut transfer)?;
        // println!("{:?}", rx_buf);

        if *val_as_bytes.last().unwrap() != 0 {
            val_as_bytes.fill_with(|| 0);
        }
        for (i, b) in rx_buf.iter().enumerate() {
            val_as_bytes[i+1] = *b;
        }

        val = u32::from_be_bytes(val_as_bytes);
        val &= mask;

        let vol = val as f32 / 4096f32 * vol_ref;
        println!("{} {:.4}", val, vol);
        stream.write(&val.to_be_bytes())?;
    }
}

fn main() -> Result<()> {
    // 192.168.137.1 or 127.0.0.1
    let mut stream = create_tcp_stream()?;
    let mut spi = create_spi()?;
    transfer_data(&mut spi, &mut stream)?;
    Ok(())
}
