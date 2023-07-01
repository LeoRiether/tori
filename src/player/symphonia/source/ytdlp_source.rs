use std::process::{Command, Stdio};

use ac_ffmpeg::format::io::IO;
use super::pcm_decoder::decode_pcm;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// TODO: get rid of unwraps 
pub fn start_player_thread(url: &str) {
    // Get urls from yt-dlp
    let ytdlp_output = Command::new("yt-dlp")
        .args(["-g", url])
        .output()
        .unwrap()
        .stdout;
    let ytdlp_output = String::from_utf8(ytdlp_output).unwrap();
    let ytdlp_urls = ytdlp_output.lines();

    eprintln!("ytdlp urls: \x1b[96m{:?}\x1b[0m", ytdlp_urls.clone().collect::<Vec<_>>());

    // Get ffmpeg mpegts stream.
    let mut ffmpeg = Command::new("ffmpeg");
    ffmpeg
        .args(["-f", "mpegts"])
        .arg("-")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .stdin(Stdio::null());
    for url in ytdlp_urls {
        ffmpeg.args(["-i", url]);
    }
    let ffmpeg = ffmpeg.spawn().unwrap();
    let stream = ffmpeg.stdout.unwrap();

    eprintln!("\x1b[96got ffmpeg stream\x1b[0m");

    let demuxer = ac_ffmpeg::format::demuxer::Demuxer::builder();
    let io = IO::from_read_stream(stream);
    let mut demuxer = demuxer.build(io).unwrap();

    let (sender, receiver) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        while let Ok(Some(packet)) = demuxer.take() {
            if let Err(e) = sender.send(packet) {
                println!("{}", e);
            }
        }
    });

    std::thread::spawn(move || {
        decode_pcm(receiver).unwrap();
    });
}
