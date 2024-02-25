// Mostly copied from symphonia-play

// Copyright (c) 2019-2022 The Project Symphonia Developers.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Platform-dependant Audio Outputs

use std::{result, time};

use super::resampler::Resampler;
use symphonia::core::audio::{AudioBufferRef, RawSample, SampleBuffer, SignalSpec};
use symphonia::core::conv::{ConvertibleSample, IntoSample};
use symphonia::core::units::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rb::*;

use log::{error, info};

pub trait AudioOutput {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()>;
    fn flush(&mut self);
}

#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
#[derive(Debug)]
pub enum AudioOutputError {
    OpenStreamError,
    PlayStreamError,
    StreamClosedError,
}

pub type Result<T> = result::Result<T, AudioOutputError>;

pub struct CpalAudioOutput;

trait AudioOutputSample:
    cpal::SizedSample + ConvertibleSample + IntoSample<f32> + RawSample + std::marker::Send + 'static
{
}

impl AudioOutputSample for f32 {}
impl AudioOutputSample for i16 {}
impl AudioOutputSample for u16 {}

impl CpalAudioOutput {
    pub fn try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn AudioOutput>> {
        // Get default host.
        let host = cpal::default_host();

        // Get the default audio output device.
        let device = match host.default_output_device() {
            Some(device) => device,
            _ => {
                error!("failed to get default audio output device");
                return Err(AudioOutputError::OpenStreamError);
            }
        };

        let config = match device.default_output_config() {
            Ok(config) => config,
            Err(err) => {
                error!("failed to get default audio output device config: {}", err);
                return Err(AudioOutputError::OpenStreamError);
            }
        };

        // Select proper playback routine based on sample format.
        match config.sample_format() {
            cpal::SampleFormat::F32 => {
                CpalAudioOutputImpl::<f32>::try_open(spec, duration, &device)
            }
            cpal::SampleFormat::I16 => {
                CpalAudioOutputImpl::<i16>::try_open(spec, duration, &device)
            }
            cpal::SampleFormat::U16 => {
                CpalAudioOutputImpl::<u16>::try_open(spec, duration, &device)
            }
            _ => unimplemented!(),
        }
    }
}

struct CpalAudioOutputImpl<T: AudioOutputSample>
where
    T: AudioOutputSample,
{
    ring_buf_producer: rb::Producer<T>,
    sample_buf: SampleBuffer<T>,
    stream: cpal::Stream,
    resampler: Option<Resampler<T>>,
}

impl<T: AudioOutputSample> CpalAudioOutputImpl<T> {
    pub fn try_open(
        spec: SignalSpec,
        duration: Duration,
        device: &cpal::Device,
    ) -> Result<Box<dyn AudioOutput>> {
        let num_channels = spec.channels.count();

        // Output audio stream config.
        let config = if cfg!(not(target_os = "windows")) {
            cpal::StreamConfig {
                channels: num_channels as cpal::ChannelCount,
                sample_rate: cpal::SampleRate(spec.rate),
                buffer_size: cpal::BufferSize::Default,
            }
        } else {
            // Use the default config for Windows.
            device
                .default_output_config()
                .expect("Failed to get the default output config.")
                .config()
        };

        // Create a ring buffer with a capacity for up-to 200ms of audio.
        let ring_len = ((200 * config.sample_rate.0 as usize) / 1000) * num_channels;

        let ring_buf = SpscRb::new(ring_len);
        let (ring_buf_producer, ring_buf_consumer) = (ring_buf.producer(), ring_buf.consumer());

        let stream_result = device.build_output_stream(
            &config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                // Write out as many samples as possible from the ring buffer to the audio
                // output.
                let written = ring_buf_consumer.read(data).unwrap_or(0);

                // Mute any remaining samples.
                data[written..].iter_mut().for_each(|s| *s = T::MID);
            },
            move |err| error!("audio output error: {}", err),
            Some(time::Duration::from_secs(1)),
        );

        if let Err(err) = stream_result {
            error!("audio output stream open error: {}", err);

            return Err(AudioOutputError::OpenStreamError);
        }

        let stream = stream_result.unwrap();

        // Start the output stream.
        if let Err(err) = stream.play() {
            error!("audio output stream play error: {}", err);

            return Err(AudioOutputError::PlayStreamError);
        }

        let sample_buf = SampleBuffer::<T>::new(duration, spec);

        let resampler = if spec.rate != config.sample_rate.0 {
            info!("resampling {} Hz to {} Hz", spec.rate, config.sample_rate.0);
            Some(Resampler::new(
                spec,
                config.sample_rate.0 as usize,
                duration,
            ))
        } else {
            None
        };

        Ok(Box::new(CpalAudioOutputImpl {
            ring_buf_producer,
            sample_buf,
            stream,
            resampler,
        }))
    }
}

impl<T: AudioOutputSample> AudioOutput for CpalAudioOutputImpl<T> {
    fn write(&mut self, decoded: AudioBufferRef<'_>) -> Result<()> {
        // Do nothing if there are no audio frames.
        if decoded.frames() == 0 {
            return Ok(());
        }

        let mut samples = if let Some(resampler) = &mut self.resampler {
            // Resampling is required. The resampler will return interleaved samples in the
            // correct sample format.
            match resampler.resample(decoded) {
                Some(resampled) => resampled,
                None => return Ok(()),
            }
        } else {
            // Resampling is not required. Interleave the sample for cpal using a sample buffer.
            self.sample_buf.copy_interleaved_ref(decoded);

            self.sample_buf.samples()
        };

        // Write all samples to the ring buffer.
        while let Some(written) = self.ring_buf_producer.write_blocking(samples) {
            samples = &samples[written..];
        }

        Ok(())
    }

    fn flush(&mut self) {
        // If there is a resampler, then it may need to be flushed
        // depending on the number of samples it has.
        if let Some(resampler) = &mut self.resampler {
            let mut remaining_samples = resampler.flush().unwrap_or_default();

            while let Some(written) = self.ring_buf_producer.write_blocking(remaining_samples) {
                remaining_samples = &remaining_samples[written..];
            }
        }

        // Flush is best-effort, ignore the returned result.
        let _ = self.stream.pause();
    }
}

pub fn _try_open(spec: SignalSpec, duration: Duration) -> Result<Box<dyn AudioOutput>> {
    CpalAudioOutput::try_open(spec, duration)
}
