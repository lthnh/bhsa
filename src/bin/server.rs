use anyhow::Result;
use num::{Complex, Num};
use num::traits::FromBytes;
use ringbuf::ring_buffer::{RbRef, RbRead, RbWrite};
use ringbuf::{Consumer, Producer, SharedRb};
use rustfft::{FftNum, FftPlanner};

use std::cmp::PartialOrd;
use std::io::{ErrorKind, Read};
use std::fmt::Display;
use std::mem;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use std::thread::JoinHandle;

use std::sync::mpsc::{self, TryRecvError};

fn create_tcp_stream() -> Result<TcpStream> {
    let address = if cfg!(unix) {
        "127.0.0.1:8080"
    } else {
        "0.0.0.0:8080"
    };
    let listener = TcpListener::bind(address)?;
    let (stream, _) = listener.accept()?;
    Ok(stream)
}

fn receive_data<T, const N: usize>(mut stream: TcpStream, tx: mpsc::Sender<T>) -> Result<JoinHandle<()>>
where
    T: FromBytes<Bytes = [u8; N]> + Send + 'static
{
    assert_eq!(N, mem::size_of::<T>());
    let receive_handle = thread::spawn(move || {
        let mut buffer = [0; N];
        loop {
            if let Err(e) = stream.read_exact(&mut buffer) {
                match e.kind() {
                    ErrorKind::UnexpectedEof => {
                        stream.shutdown(Shutdown::Both).unwrap();
                        break;
                    },
                    _ => eprintln!("{e}")
                }
            }
            let val = T::from_le_bytes(&buffer);
            tx.send(val).unwrap();
            // println!("{}", value);
        };
    });
    Ok(receive_handle)
}

fn record_data<T, U>(rx: mpsc::Receiver<T>, mut prod: Producer<T, U>) -> Result<JoinHandle<()>>
where
    T: Send + 'static,
    U: RbRef + Send + 'static,
    <U as RbRef>::Rb: RbWrite<T>
{
     let record_handle = thread::spawn(move || {
        let mut count;
        let mut buffer: Vec<T> = Vec::new();
        loop {
            count = 0;
            while !buffer.is_empty() {
                if let Some(v) = buffer.pop() {
                    match prod.push(v) {
                        Ok(_) => continue,
                        Err(v) => {
                            buffer.push(v);
                            count += 1;
                        }
                    }
                    if count == 3 {
                        break;
                    }
                }
            }
            match rx.try_recv() {
                Ok(v) => {
                    // println!("Received {}", v);
                    if let Err(v) = prod.push(v) {
                        buffer.push(v);
                    };
                }
                Err(e) => {
                        if e == TryRecvError::Disconnected {
                            println!("Stop sending value");
                            break;
                        }
                    }
                }
            }
        }
    );
    Ok(record_handle)
}

fn process_data<T, U, const N: usize>(mut cons: Consumer<T, U>) -> Result<JoinHandle<()>>
where
    T: FftNum + PartialOrd + Display,
    U: RbRef + Send + 'static,
    <U as RbRef>::Rb: RbRead<T>
{
    let process_handle = thread::spawn(move || {
        let mut buffer: Vec<T> = Vec::with_capacity(N);
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(N);
        loop {
            if cons.len() >= N {
                cons.pop_slice(&mut buffer);
                let mut another_buffer = real_to_complex(&buffer);
                fft.process(&mut another_buffer);
                for value in another_buffer {
                    println!("{}", value);
                }
            }
        }
    });
    Ok(process_handle)
}

#[inline(always)]
fn real_to_complex<T>(input_slice: &[T]) -> Vec<Complex<T>>
where
    T: Num + Copy
{
    let mut output = Vec::new();
    for elem in input_slice {
        let value = Complex::new(*elem, T::zero());
        output.push(value);
    }
    output
}

fn main() -> Result<()> {
    let stream = create_tcp_stream()?;
    let (tx, rx) = mpsc::channel();
    let rb = SharedRb::<f32, Vec<_>>::new(1000);
    let (prod, cons) = rb.split();

    let receive_handle = receive_data::<_, { mem::size_of::<f32>() }>(stream, tx)?;
    let record_handle = record_data(rx, prod)?;
    let process_handle = process_data::<_, _, 1000>(cons)?;

    receive_handle.join().unwrap();
    record_handle.join().unwrap();
    process_handle.join().unwrap();

    Ok(())
}