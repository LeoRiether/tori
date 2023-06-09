use std::{
    error::Error as StdError,
    fs::File,
    io::{self, Read, Write},
    mem,
    path::PathBuf,
    process::Stdio,
    result::Result as StdResult,
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

use crate::{config::Config, error::Result};

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
noise_reduction = 0.3
"#
    };
}

static MAX_BAR_VALUE: u16 = ((1 << 16_u32) - 1) as u16;

pub struct CavaOptions {
    pub bars: usize,
}

type ThreadResult = StdResult<(), Box<dyn StdError + Send + Sync>>;

#[derive(Debug)]
pub enum ThreadHandle {
    Stopped(ThreadResult),
    Running(thread::JoinHandle<ThreadResult>),
}

impl Default for ThreadHandle {
    fn default() -> Self {
        Self::Stopped(Ok(()))
    }
}

pub struct Visualizer {
    tmp_path: PathBuf,
    data: Arc<Mutex<Vec<u16>>>,
    stop_flag: Arc<AtomicBool>,
    handle: ThreadHandle,
}

impl Visualizer {
    pub fn new(opts: CavaOptions) -> Result<Self> {
        let tmp_path = tori_tempfile(&opts)?;

        let mut process = std::process::Command::new("cava")
            .arg("-p")
            .arg(&tmp_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| {
                format!("Failed to spawn the visualizer process. Is `cava` installed? The received error was: {}", e)
            })?;

        let data = Arc::new(Mutex::new(vec![0_u16; opts.bars]));
        let stop_flag = Arc::new(AtomicBool::new(false));
        let handle: thread::JoinHandle<ThreadResult>;

        {
            let data = data.clone();
            let stop_flag = stop_flag.clone();
            handle = thread::spawn(move || {
                let mut buf = vec![0_u8; 2 * opts.bars];
                while !stop_flag.load(atomic::Ordering::Relaxed) {
                    let stdout = process.stdout.as_mut().unwrap();
                    let read_res = stdout.read_exact(&mut buf);

                    if let Err(e) = read_res {
                        let mut stderr_contents = String::new();
                        let stderr = process.stderr.as_mut().unwrap();
                        stderr.read_to_string(&mut stderr_contents).unwrap();
                        return Err(format!("'{}'. Process stderr: {}", e, stderr_contents).into());
                    }

                    let mut data = data.lock().unwrap();
                    for i in 0..data.len() {
                        data[i] = u16::from_le_bytes([buf[2 * i], buf[2 * i + 1]]);
                    }
                }
                process.kill()?;
                Ok(())
            });
        }

        Ok(Self {
            tmp_path,
            data,
            stop_flag,
            handle: ThreadHandle::Running(handle),
        })
    }

    pub fn render(&self, buffer: &mut tui::buffer::Buffer) {
        let lerp = |from: u8, to: u8, perc: f64| {
            (from as f64 + perc * (to as f64 - from as f64)).round() as u8
        };
        let lerp_grad = |gradient: [(u8, u8, u8); 2], perc| {
            Color::Rgb(
                lerp(gradient[0].0, gradient[1].0, perc),
                lerp(gradient[0].1, gradient[1].1, perc),
                lerp(gradient[0].2, gradient[1].2, perc),
            )
        };

        let gradient = Config::global().visualizer_gradient;

        let data = self.data.lock().unwrap();
        let columns = std::cmp::min(data.len(), buffer.area().width as usize / 2);
        let size = *buffer.area();
        for i in 0..columns {
            let perc = i as f64 / columns as f64;
            let style = Style::default().bg(lerp_grad(gradient, perc));
            let height = (data[i] as u64 * size.height as u64 / MAX_BAR_VALUE as u64) as u16;

            let area = Rect {
                x: 2 * i as u16,
                y: size.height.saturating_sub(height),
                width: 1,
                height,
            };
            buffer.set_style(area, style);
        }
    }

    pub fn thread_handle(&mut self) -> &ThreadHandle {
        match mem::take(&mut self.handle) {
            ThreadHandle::Running(handle) if handle.is_finished() => {
                self.handle = ThreadHandle::Stopped(handle.join().unwrap());
            }
            other => {
                self.handle = other;
            }
        }

        &self.handle
    }
}

impl Drop for Visualizer {
    fn drop(&mut self) {
        // ~~hopefully~~ stop thread execution
        self.stop_flag.store(true, atomic::Ordering::Relaxed);
        std::fs::remove_file(&self.tmp_path).ok();
    }
}

fn tori_tempfile(opts: &CavaOptions) -> StdResult<PathBuf, std::io::Error> {
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
