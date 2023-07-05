use std::error::Error;
use std::fmt;
use std::result::Result as StdResult;

type Result<T> = StdResult<T, MpvError>;

macro_rules! define_method {
    (fn $name:ident($( $id:ident: $ty:ty ),*) -> $ret:ty) => {
        pub fn $name(&self, $($id: $ty),*) -> Result<$ret> {
            match self {
                Self::V034(mpv) => Ok(mpv.$name($($id),*)?),
                Self::V035(mpv) => Ok(mpv.$name($($id),*)?),
            }
        }
    };
}

macro_rules! define_data {
    (set $name:ident $typ:ty) => {
        pub fn $name(&self, name: &str, data: $typ) -> Result<()> {
            match self {
                Self::V034(initializer) => Ok(initializer.set_property(name, data)?),
                Self::V035(initializer) => Ok(initializer.set_property(name, data)?),
            }
        }
    };
    (get $name:ident $typ:ty) => {
        pub fn $name(&self, name: &str) -> Result<$typ> {
            match self {
                Self::V034(initializer) => Ok(initializer.get_property::<$typ>(name)?),
                Self::V035(initializer) => Ok(initializer.get_property::<$typ>(name)?),
            }
        }
    };
    (add $name:ident $typ:ty) => {
        pub fn $name(&self, name: &str, data: $typ) -> Result<()> {
            match self {
                Self::V034(initializer) => Ok(initializer.add_property(name, data)?),
                Self::V035(initializer) => Ok(initializer.add_property(name, data)?),
            }
        }
    };
}

/// A wrapper around mpv v0.34 or mpv v0.35, depending on which is installed.
/// Uses libmpv-rs or libmpv-sirno depending on the mpv version.
pub enum Mpv {
    V034(mpv034::Mpv),
    V035(mpv035::Mpv),
}

impl Mpv {
    pub fn with_initializer<F>(init: F) -> Result<Self>
    where
        F: FnOnce(MpvInitializer) -> Result<()>,
    {
        let api_version = unsafe { libmpv_sys::mpv_client_api_version() };

        if api_version >> 16 == mpv035::MPV_CLIENT_API_MAJOR {
            let mpv = mpv035::Mpv::with_initializer(|mpv| {
                let initializer = MpvInitializer::V035(mpv);
                init(initializer).map_err(|e| e.unwrap_v035())?;
                Ok(())
            })?;
            Ok(Self::V035(mpv))
        } else {
            let mpv = mpv034::Mpv::with_initializer(|mpv| {
                let initializer = MpvInitializer::V034(mpv);
                init(initializer).map_err(|e| e.unwrap_v034())?;
                Ok(())
            })?;
            Ok(Self::V034(mpv))
        }
    }

    pub fn play(&self, path: &str) -> Result<()> {
        match self {
            Self::V034(mpv) => {
                mpv.playlist_load_files(&[(path, mpv034::FileState::Replace, None)])?;
                Ok(())
            }
            Self::V035(mpv) => {
                mpv.playlist_load_files(&[(path, mpv035::FileState::Replace, None)])?;
                Ok(())
            }
        }
    }

    pub fn queue(&self, path: &str) -> Result<()> {
        match self {
            Self::V034(mpv) => {
                mpv.playlist_load_files(&[(path, mpv034::FileState::AppendPlay, None)])?;
                Ok(())
            }
            Self::V035(mpv) => {
                mpv.playlist_load_files(&[(path, mpv035::FileState::AppendPlay, None)])?;
                Ok(())
            }
        }
    }

    define_method! { fn seek_forward(s: f64) -> () }
    define_method! { fn seek_backward(s: f64) -> () }
    define_method! { fn seek_percent_absolute(p: usize) -> () }
    define_method! { fn playlist_next_weak() -> () }
    define_method! { fn playlist_previous_weak() -> () }
    define_method! { fn command(name: &str, args: &[&str]) -> () }

    define_data! { get get_bool bool }
    define_data! { get get_str String }
    define_data! { get get_i64 i64 }

    define_data! { set set_str &str }
    define_data! { set set_i64 i64 }

    define_data! { add add_isize isize }
}

/// Wrapper around the other MpvInitializers
pub enum MpvInitializer {
    V034(mpv034::MpvInitializer),
    V035(mpv035::MpvInitializer),
}

impl MpvInitializer {
    define_data! { set set_bool bool }
    define_data! { set set_i64 i64 }
    define_data! { set set_str &str }
}

/// Wrapper around an mpv error
#[derive(Debug, Clone)]
pub enum MpvError {
    V034(mpv034::Error),
    V035(mpv035::Error),
}

impl MpvError {
    pub fn unwrap_v034(self) -> mpv034::Error {
        match self {
            Self::V034(err) => err,
            Self::V035(_) => panic!("Expected mpv v0.34, found v0.35"),
        }
    }

    pub fn unwrap_v035(self) -> mpv035::Error {
        match self {
            Self::V034(_) => panic!("Expected mpv v0.35, found v0.34"),
            Self::V035(err) => err,
        }
    }
}

impl From<mpv034::Error> for MpvError {
    fn from(err: mpv034::Error) -> Self {
        Self::V034(err)
    }
}

impl From<mpv035::Error> for MpvError {
    fn from(err: mpv035::Error) -> Self {
        Self::V035(err)
    }
}

impl fmt::Display for MpvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V034(err) => err.fmt(f),
            Self::V035(err) => err.fmt(f),
        }
    }
}

impl Error for MpvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::V034(err) => Some(err),
            Self::V035(err) => Some(err),
        }
    }
}
