//! Mostly copied from 
//! https://gist.github.com/Lighty0410/ed2d452c3cb318b974170e37ce4bbe52
//! and https://github.com/pdeljanov/Symphonia/issues/210#issuecomment-1569987145

use std::error::Error;
use std::sync::mpsc::Receiver;

use symphonia::core::audio::Layout;
use symphonia::core::codecs::{CodecParameters, Decoder, CODEC_TYPE_PCM_S16LE};
use symphonia::core::formats::Packet;
use symphonia::default::codecs::PcmDecoder;

use crate::player::symphonia::output::CpalAudioOutput;

pub fn decode_pcm(
    receiver: Receiver<ac_ffmpeg::packet::Packet>,
) -> Result<(), Box<dyn Error>> {
    let mut pcm_decoder = PcmDecoder::try_new(
        &CodecParameters {
            codec: CODEC_TYPE_PCM_S16LE,
            sample_rate: Some(16000),
            time_base: None,
            n_frames: None,
            start_ts: 0,
            sample_format: None,
            bits_per_sample: None,
            bits_per_coded_sample: Some(16),
            channels: None,
            channel_layout: Some(Layout::Stereo),
            delay: None,
            padding: None,
            max_frames_per_packet: Some(3000),
            packet_data_integrity: false,
            verification_check: None,
            frames_per_block: None,
            extra_data: None,
        },
        &Default::default(),
    )?;

    let mut audio_output = None;
    while let Ok(ac_packet) = receiver.recv() {
        let packet = Packet::new_from_slice(0, 0, 0, ac_packet.data());

        match pcm_decoder.decode(&packet) {
                Ok(decoded) => {
                    // If the audio output is not open, try to open it.
                    if audio_output.is_none() {
                        // Get the audio buffer specification. This is a description of the decoded
                        // audio buffer's sample format and sample rate.
                        let spec = *decoded.spec();

                        // Get the capacity of the decoded buffer. Note that this is capacity, not
                        // length! The capacity of the decoded buffer is constant for the life of the
                        // decoder, but the length is not.
                        let duration = decoded.capacity() as u64;

                        // Try to open the audio output.
                        audio_output.replace(CpalAudioOutput::try_open(spec, duration).unwrap());
                    } else {
                        // TODO: Check the audio spec. and duration hasn't changed.
                    }

                    // Write the decoded audio samples to the audio output if the presentation timestamp
                    // for the packet is >= the seeked position (0 if not seeking).
                    if let Some(audio_output) = audio_output.as_mut() {
                        audio_output.write(decoded).unwrap()
                    }
                }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }

    Ok(())
}
