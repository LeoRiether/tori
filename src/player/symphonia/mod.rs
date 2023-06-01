use crate::error::Result;
use cpal::{traits::HostTrait, Device, Stream};
use symphonia::core::{
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymError,
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

pub struct SymphoniaPlayer {
    device: Device,
    stream: Option<Stream>,
}

impl super::Player for SymphoniaPlayer {
    fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("Couldn't open default audio device!")?;

        Ok(Self {
            device,
            stream: None,
        })
    }

    fn play(&mut self, path: &str) -> Result<()> {
        // Open the media source.
        let src = std::fs::File::open(path).expect("failed to open media");

        // Create the media source stream.
        let mss = MediaSourceStream::new(Box::new(src), Default::default());

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let hint = Hint::default();

        // Probe the media source.
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .expect("unsupported format");

        // Get the instantiated format reader.
        let mut format = probed.format;

        // Find the first audio track with a known (decodeable) codec.
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .expect("no supported audio tracks");

        // Use the default options for the decoder.
        let dec_opts: DecoderOptions = Default::default();

        // Create a decoder for the track.
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .expect("unsupported codec");

        // Store the track identifier, it will be used to filter packets.
        let track_id = track.id;

        // The decode loop.
        loop {
            // Get the next packet from the media format.
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymError::ResetRequired) => {
                    // The track list has been changed. Re-examine it and create a new set of decoders,
                    // then restart the decode loop. This is an advanced feature and it is not
                    // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                    // for chained OGG physical streams.
                    unimplemented!();
                }
                Err(err) => {
                    // A unrecoverable error occurred, halt decoding.
                    panic!("{}", err);
                }
            };

            // Consume any new metadata that has been read since the last packet.
            while !format.metadata().is_latest() {
                // Pop the old head of the metadata queue.
                format.metadata().pop();

                // Consume the new metadata at the head of the metadata queue.
            }

            // If the packet does not belong to the selected track, skip over it.
            if packet.track_id() != track_id {
                continue;
            }

            // Decode the packet into audio samples.
            match decoder.decode(&packet) {
                Ok(decoded) => {
                    // Consume the decoded audio samples (see below).
                }
                Err(SymError::IoError(_)) => {
                    // The packet failed to decode due to an IO error, skip the packet.
                    continue;
                }
                Err(SymError::DecodeError(_)) => {
                    // The packet failed to decode due to invalid data, skip the packet.
                    continue;
                }
                Err(err) => {
                    // An unrecoverable error occurred, halt decoding.
                    panic!("{}", err);
                }
            }
        }
    }

    fn queue(&mut self, path: &str) -> Result<()> {
        todo!()
    }

    fn seek(&mut self, seconds: f64) -> Result<()> {
        todo!()
    }

    fn seek_absolute(&mut self, percent: usize) -> Result<()> {
        todo!()
    }

    fn playlist_next(&mut self) -> Result<()> {
        todo!()
    }

    fn playlist_previous(&mut self) -> Result<()> {
        todo!()
    }

    fn toggle_pause(&mut self) -> Result<()> {
        todo!()
    }

    fn toggle_loop_file(&mut self) -> Result<()> {
        todo!()
    }

    fn looping_file(&self) -> Result<bool> {
        todo!()
    }

    fn volume(&self) -> Result<i64> {
        todo!()
    }

    fn add_volume(&mut self, x: isize) -> Result<()> {
        todo!()
    }

    fn set_volume(&mut self, x: i64) -> Result<()> {
        todo!()
    }

    fn toggle_mute(&mut self) -> Result<()> {
        todo!()
    }

    fn muted(&self) -> Result<bool> {
        todo!()
    }

    fn media_title(&self) -> Result<String> {
        todo!()
    }

    fn percent_pos(&self) -> Result<i64> {
        todo!()
    }

    fn time_pos(&self) -> Result<i64> {
        todo!()
    }

    fn time_remaining(&self) -> Result<i64> {
        todo!()
    }

    fn paused(&self) -> Result<bool> {
        todo!()
    }

    fn shuffle(&mut self) -> Result<()> {
        todo!()
    }

    fn playlist_count(&self) -> Result<usize> {
        todo!()
    }

    fn playlist_track_title(&self, i: usize) -> Result<String> {
        todo!()
    }

    fn playlist_position(&self) -> Result<usize> {
        todo!()
    }
}
