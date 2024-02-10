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
use tui::{layout::Rect, prelude::*, style::Style};

use crate::{color::Color, config::Config, error::Result};

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

#[derive(Default)]
pub struct Visualizer(pub Option<VisualizerState>);

impl Visualizer {
    pub fn toggle(&mut self, width: usize) -> Result<()> {
        if self.0.take().is_none() {
            self.0 = Some(VisualizerState::new(width)?);
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        if let Some(mut state) = self.0.take() {
            match state.thread_handle() {
                ThreadHandle::Stopped(Ok(())) => {
                    self.0 = None;
                }
                ThreadHandle::Stopped(Err(e)) => {
                    self.0 = None;
                    return Err(format!("The visualizer process exited with error: {}", e).into());
                }
                _ => {
                    self.0 = Some(state);
                }
            }
        }
        Ok(())
    }

    pub fn render(&self, _: Rect, buffer: &mut Buffer) {
        if self.0.is_none() {
            return;
        }

        let state = self.0.as_ref().unwrap();

        let gradient = &Config::global().visualizer_gradient[..];
        let data = state.data.lock().unwrap();
        let columns = std::cmp::min(data.len(), buffer.area().width as usize / 2);
        let size = *buffer.area();
        for i in 0..columns {
            let perc = i as f64 / columns as f64;
            let style = Style::default().bg(Color::lerp_many(gradient, perc).into());
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
}

pub struct VisualizerState {
    tmp_path: PathBuf,
    data: Arc<Mutex<Vec<u16>>>,
    stop_flag: Arc<AtomicBool>,
    handle: ThreadHandle,
}

impl VisualizerState {
    pub fn new(width: usize) -> Result<Self> {
        let opts = CavaOptions { bars: width / 2 };
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

impl Drop for VisualizerState {
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
