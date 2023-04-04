use std::{
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
    process::Stdio,
    sync::{
        atomic::{self, AtomicBool},
        Arc, Mutex,
    },
    thread,
};

use rand::{thread_rng, Rng};
use tui::{
    layout::Rect,
    style::{Color, Style},
};

macro_rules! cava_config {
    () => {
        r#"
[general]
bars = {0}
framerate = 45
lower_cutoff_freq = 30
higher_cutoff_freq = 16000

[output]
method = raw
channels = mono
data_format = binary
bit_format = 16bit
reverse = 0

[smoothing]
; monstercat = 1
noise_reduction = 0.33
"#
    };
}

static MAX_BAR_VALUE: u16 = ((1 << 16_u32) - 1) as u16;

pub struct CavaOptions {
    pub bars: usize,
}

pub struct Visualizer {
    tmp_path: PathBuf,
    data: Arc<Mutex<Vec<u16>>>,
    stop_flag: Arc<AtomicBool>,
}

impl Visualizer {
    pub fn new(opts: CavaOptions) -> Result<Self, Box<dyn Error>> {
        let tmp_path = tori_tempfile(&opts)?;

        let mut process = std::process::Command::new("cava")
            .arg("-p")
            .arg(&tmp_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .spawn()?;

        let data = Arc::new(Mutex::new(vec![0; opts.bars]));
        let stop_flag = Arc::new(AtomicBool::new(false));

        {
            let data = data.clone();
            let stop_flag = stop_flag.clone();
            thread::spawn(move || {
                let mut buf = vec![0_u8; 2 * opts.bars];
                while !stop_flag.load(atomic::Ordering::Relaxed) {
                    let stdout = process.stdout.as_mut().unwrap();
                    stdout.read_exact(&mut buf).unwrap();

                    let mut data = data.lock().unwrap();
                    for i in 0..data.len() {
                        data[i] = u16::from_le_bytes([buf[2 * i], buf[2 * i + 1]]);
                    }
                }
                process.kill().ok();
            });
        }

        Ok(Self {
            tmp_path,
            data,
            stop_flag,
        })
    }

    pub fn render(&self, buffer: &mut tui::buffer::Buffer) {
        let lerp = |from: u8, to: u8, perc: f64| {
            (perc * (to as f64) + (1. - perc) * (from as f64)).round() as u8
        };
        let lerp_rgb = |from: (u8, u8, u8), to: (u8, u8, u8), perc| {
            Color::Rgb(
                lerp(from.0, to.0, perc),
                lerp(from.1, to.1, perc),
                lerp(from.2, to.2, perc),
            )
        };

        // let left_color = (37, 28, 156);
        // let right_color = (104, 27, 171);
        let left_color = (40, 17, 105);
        let right_color = (21, 71, 133);

        let data = self.data.lock().unwrap();
        let columns = std::cmp::min(data.len(), buffer.area().width as usize);
        let size = *buffer.area();
        for i in 0..columns {
            let perc = i as f64 / columns as f64;
            let style = Style::default().bg(lerp_rgb(left_color, right_color, perc));
            let height = (data[i] as u64 * size.height as u64 / MAX_BAR_VALUE as u64) as u16;

            let area = Rect {
                x: i as u16,
                y: size.height.saturating_sub(height),
                width: 1,
                height,
            };
            buffer.set_style(area, style);
        }
    }
}

impl Drop for Visualizer {
    // See: https://stackoverflow.com/a/42791007
    fn drop(&mut self) {
        // ~~hopefully~~ stop thread execution
        self.stop_flag.store(true, atomic::Ordering::Relaxed);
        std::fs::remove_file(&self.tmp_path).ok();
    }
}

fn tori_tempfile(opts: &CavaOptions) -> Result<PathBuf, std::io::Error> {
    let path = std::env::temp_dir().join(format!("tori-{:x}", thread_rng().gen::<u32>()));

    if path.is_file() {
        return Err(io::ErrorKind::AlreadyExists.into());
    }

    let cava_config = format!(cava_config!(), opts.bars);

    let mut temp = File::create(&path)?;
    temp.write_all(cava_config.as_bytes()).unwrap();
    temp.flush().unwrap();

    Ok(path)
}
