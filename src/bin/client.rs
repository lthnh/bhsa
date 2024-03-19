use std::error::Error;
use std::io::Write;
use std::net::TcpStream;

fn main() -> Result<(), Box<dyn Error>>{
    let mut stream = TcpStream::connect("192.168.137.1:8080")?;
    for i in 1.. {
        let x = (i as f32).to_radians().sin().to_be_bytes();
        // let x = (i as f32).to_be_bytes();
        stream.write_all(&x)?;
    }
    // let val = 3f32;
    // stream.write(&val.to_le_bytes())?;
    Ok(())
}