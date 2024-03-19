use anyhow::Result;
use spidev::{Spidev, SpidevOptions, SpidevTransfer, SpiModeFlags};

const CLOCK_FREQUENCY: u32 = 1_800_000;
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

fn full_duplex(spi: &mut Spidev) -> Result<()> {
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
        println!("{:.4}", vol);
    }
}

fn main() -> Result<()> {
    let mut spi = create_spi()?;
    full_duplex(&mut spi)?;
    Ok(())
}