use crate::error::Result;
use std::{
    fs::File,
    io,
    path::Path,
    process::{Command, Stdio},
    thread,
};

use symphonia::core::{
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    errors::Error as SymError,
    formats::FormatOptions,
    io::{MediaSource, MediaSourceStream, ReadOnlySource},
    meta::MetadataOptions,
    probe::Hint,
};

use crate::player::symphonia::output::CpalAudioOutput;

// TODO: remove `expects` and `unwraps`
pub fn start_player_thread(path: &str) {
    let (mss, hint) = mss_from_path(path).unwrap();

    // Use the default options for metadata and format readers.
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };

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

    // Spawn a thread to read packets from the format reader and send them to the decoder.
    // TODO: if the user pauses the player, this thread continues to run. Should this really be the
    // case?
    thread::spawn(move || {
        let mut audio_output = None;
        loop {
            let packet = match format.next_packet() {
                Ok(packet) => packet,
                Err(SymError::ResetRequired) => {
                    // The track list has been changed. Re-examine it and create a new set of decoders,
                    // then restart the decode loop. This is an advanced feature and it is not
                    // unreasonable to consider this "the end." As of v0.5.0, the only usage of this is
                    // for chained OGG physical streams.
                    unimplemented!();
                }
                Err(SymError::IoError(e))
                    if e.kind() == io::ErrorKind::UnexpectedEof
                        && e.to_string() == "end of stream" =>
                {
                    // File ended
                    break;
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
                eprintln!("Got new metadata! {:?}", format.metadata().current());
            }

            // If the packet does not belong to the selected track, skip over it.
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
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
    });
}

fn mss_from_path(mut path: &str) -> Result<(MediaSourceStream, Hint)> {
    let mut force_ytdlp = false;
    if let Some(url) = path.strip_prefix("ytdlp://") {
        path = url;
        force_ytdlp = true;
    }

    let mut hint = Hint::default();
    let src: Box<dyn MediaSource> =
        if force_ytdlp || path.starts_with("http://") || path.starts_with("https://") {
            // Get urls from yt-dlp
            let ytdlp_output = Command::new("yt-dlp")
                .args(["-g", path])
                .output()
                .unwrap()
                .stdout;
            let ytdlp_output = String::from_utf8(ytdlp_output).unwrap();
            let ytdlp_urls = ytdlp_output.lines();

            // Get ffmpeg mpegts stream.
            let mut ffmpeg = Command::new("ffmpeg");
            for url in ytdlp_urls {
                ffmpeg.args(["-i", url]);
            }
            ffmpeg
                .args(["-f", "mp3"]) // FIXME: don't do this. If you know how to do better please tell me how. Symphonia still doesn't support opus afaik.
                .arg("-")
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .stdin(Stdio::null());
            let mut ffmpeg = ffmpeg.spawn().unwrap();
            let src = ffmpeg.stdout.take().unwrap();

            hint.with_extension("mp3");
            Box::new(ReadOnlySource::new(src))
        } else {
            if let Some(ext) = Path::new(path).extension().and_then(|s| s.to_str()) {
                hint.with_extension(ext);
            }
            Box::new(File::open(path).expect("failed to open media"))
        };

    let mss = MediaSourceStream::new(src, Default::default());
    Ok((mss, hint))
}
