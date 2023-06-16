mod file_source;
mod ytdlp_source;

use super::output::CpalAudioOutput;
use crate::error::Result;
use crossbeam_channel::Receiver;
use std::{
    fs::File,
    io,
    path::Path,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
};
use symphonia::core::{
    audio::{AudioBuffer, AudioBufferRef},
    codecs::{Decoder, DecoderOptions, CODEC_TYPE_NULL},
    errors::{Error as SymError, Result as SymResult},
    formats::{FormatOptions, Packet},
    io::{MediaSource, MediaSourceStream, ReadOnlySource},
    meta::MetadataOptions,
    probe::Hint,
};

#[derive(Debug, Default)]
pub struct Controller {}

impl Controller {
    pub fn play(&mut self, path: &str) -> Result<()> {
        start_player_thread(path);
        Ok(())
    }
}

fn start_player_thread(mut path: &str) {
    let mut force_ytdlp = false;
    if let Some(url) = path.strip_prefix("ytdlp://") {
        path = url;
        force_ytdlp = true;
    }

    if path.starts_with("http://") || path.starts_with("https://") || force_ytdlp {
        ytdlp_source::start_player_thread(path);
    } else {
        file_source::start_player_thread(path);
    }
}
