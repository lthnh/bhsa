use std::path::Path;
use std::sync::Arc;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SizedSample};
use fundsp::audionode::AudioNode;
use fundsp::hacker32::{Wave32, Wave32Player};

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .unwrap();
    let config = device.default_output_config().unwrap();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()).unwrap(),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()).unwrap(),
        _ => panic!("Unsupported format!")
    }
}
fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> Result<(), anyhow::Error>
where
    T: SizedSample + FromSample<f32>
{
    let file_path = Path::new("ICBHI_final_database/101_1b1_Al_sc_Meditron.wav").as_os_str();
    let sound = Wave32::load(file_path).unwrap();

    let end_len = sound.len();
    let mut player: Wave32Player<f32> = Wave32Player::new(&Arc::new(sound), 0, 0, end_len, None);


    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    player.set_sample_rate(sample_rate.into());

    let mut next_value = move | | player.get_stereo();

    let err_fn = |err| eprintln!("An error occurred on stream: {}", err);

    let stream = device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            write_data(data, channels, &mut next_value);
        },
        err_fn,
        None
    )?;
    stream.play()?;

    Ok(())
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> (f32, f32))
where
    T: SizedSample + FromSample<f32>
{
    for frame in output.chunks_mut(channels) {
        let sample = next_sample();
        let left = T::from_sample(sample.0);
        let right = T::from_sample(sample.1);

        for (channel, sample) in frame.iter_mut().enumerate() {
            if channel == 0 {
                *sample = left;
            } else {
                *sample = right;
            }
        }
    }
}
