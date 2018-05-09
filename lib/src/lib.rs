extern crate rustc_version;

use std::env;
use std::fmt::Display;
use std::process;
use std::str::FromStr;
use std::time::{UNIX_EPOCH, SystemTime, SystemTimeError, Duration};

pub use rustc_version::Version;

macro_rules! __make {
    ($(($varname:expr, $fieldname:ident, $field_ty:ty, $prepare:expr, $parse:expr,));*; ) => {

#[derive(Debug)]
pub struct BuildInfo {
    $(
        $fieldname: $field_ty,
    )*
}

impl BuildInfo {
    #[doc(hidden)]
    pub fn new(
        $(
            $fieldname: Option<&'static str>,
        )*
    ) -> Self {
        $(
            let $fieldname: $field_ty = $parse($fieldname);
        )*

        Self {
            $(
                $fieldname,
            )*
        }
    }
}

/// This function creates some new environment variables which can be used
/// by `buildinfo!` macro, it's intended to be used in `build.rs` script.
pub fn prepare() {
    $(
        print_env($varname, $prepare);
    )*
}

/// This macro can be used to obtain `BuildInfo` instance in compile time.
#[macro_export]
macro_rules! buildinfo {
    () => {
        $crate::BuildInfo::new(
            $(
                __buildinfo_var!($varname),
            )*
        )
    };
}

    }
}

__make!(
    (
        "TARGET_TRIPLE", target_triple, &'static str,
        Some(var("TARGET")),
        |x| Option::unwrap(x),
    );
    (
        "HOST_TRIPLE", host_triple, &'static str,
        Some(var("HOST")),
        |x| Option::unwrap(x),
    );
    (
        "OPT_LEVEL", opt_level, &'static str,
        Some(var("OPT_LEVEL")),
        |x| Option::unwrap(x),

    );
    (
        "DEBUG", debug, bool,
        Some(var("DEBUG")),
        |x| bool::from_str(Option::unwrap(x)).expect("buildinfo debug"),
    );
    (
        "PROFILE", profile, &'static str,
        Some(var("PROFILE")),
        |x| Option::unwrap(x),
    );

    (
        "RUSTC_VERSION", rustc_version, Version,
        Some(rustc_version::version().expect("buildinfo prepare rustc_version")),
        |x| Version::parse(Option::unwrap(x)).expect("buildinfo build rustc_version"),
    );
    (
        "COMPILED_AT", compiled_at, SystemTime,
        Some(now().expect("buildinfo prepare now")),
        |x| UNIX_EPOCH + Duration::from_secs(
                u64::from_str(Option::unwrap(x)).expect("buildinfo build now")
            ),
    );
    (
        "GIT_COMMIT", git_commit, Option<&'static str>,
        git_commit().ok(),
        |x| x,
    );
    (
        "HOSTNAME", hostname, &'static str,
        ffi::hostname().ok(),
        |x| Option::unwrap(x),
    );
    (
        "USERNAME", username, &'static str,
        ffi::username().ok(),
        |x| Option::unwrap(x),
    );
);

impl BuildInfo {
    /// The target triple that is being compiled for. Some more information about target
    /// triples can be found in
    /// [clang’s own documentation](http://clang.llvm.org/docs/CrossCompilation.html#target-triple).
    pub fn target_triple(&self) -> &str {
        self.target_triple
    }

    /// The host triple of the rust compiler.
    pub fn host_triple(&self) -> &str {
        self.host_triple
    }

    /// `opt-level` compiler option.
    pub fn opt_level(&self) -> &str {
        self.opt_level
    }

    /// Is debug information included.
    pub fn debug(&self) -> bool {
        self.debug
    }

    /// `release` for release builds, `debug` for other builds.
    pub fn profile(&self) -> &str {
        self.profile
    }

    /// Version of rustc compiler.
    pub fn rustc_version(&self) -> &Version {
        &self.rustc_version
    }

    /// Compilation Unix time.
    pub fn compiled_at(&self) -> SystemTime {
        self.compiled_at
    }

    /// Latest Git commit.
    pub fn git_commit(&self) -> Option<&str> {
        self.git_commit
    }

    /// Build hostname.
    pub fn hostname(&self) -> &str {
        self.hostname
    }

    /// Build username.
    pub fn username(&self) -> &str {
        self.username
    }
}

fn print_env<K: Display, V: Display>(key: K, value: Option<V>) {
    if let Some(value) = value {
        println!("cargo:rustc-env=BUILD_INFO_{}={}", key, value)
    }
}

fn var<K: AsRef<str>>(key: K) -> String {
    if let Ok(value) = env::var(key.as_ref()) {
        value
    } else {
        eprintln!("Missing environment variable `{}`", key.as_ref());
        ::std::process::exit(1)
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! __buildinfo_var {
    ($name:expr) => {
        option_env!(concat!("BUILD_INFO_", $name))
    };
}

fn now() -> Result<u64, SystemTimeError> {
    let now = SystemTime::now();
    let elapsed = now.duration_since(UNIX_EPOCH)?;
    Ok(elapsed.as_secs())
}

fn git_commit() -> Result<String, ::std::io::Error> {
    let output = process::Command::new("git")
        .arg("rev-parse").arg("--verify").arg("HEAD")
        .output()?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout);
        if !hash.is_empty() {
            Ok(hash.to_string())
        } else {
            Err(::std::io::Error::new(::std::io::ErrorKind::UnexpectedEof, "empty hash"))
        }
    } else {
        Err(::std::io::Error::new(::std::io::ErrorKind::Other, "git failed"))
    }
}

#[cfg(unix)]
mod ffi {
    extern crate libc;

    use std::ffi::CStr;

    pub fn hostname() -> Result<String, ::std::io::Error> {
        let mut buffer = [0i8; 256];
        unsafe {
            if libc::gethostname(buffer.as_mut_ptr(), buffer.len()) == -1 {
                return Err(::std::io::Error::last_os_error())
            }
            Ok(CStr::from_ptr(buffer.as_ptr()).to_string_lossy().into_owned())
        }
    }

    pub fn username() -> Result<String, ::std::io::Error> {
        unsafe {
            let uid = libc::geteuid();
            let passwd = libc::getpwuid(uid);
            if passwd.is_null() {
                return Err(::std::io::Error::last_os_error())
            }
            Ok(CStr::from_ptr((*passwd).pw_name).to_string_lossy().into_owned())
        }
    }
}

#[cfg(windows)]
mod ffi {
    extern crate winapi;

    use std::iter::FromIterator;

    use self::winapi::um::winbase::{GetComputerNameA, GetUserNameA};
    use self::winapi::um::winnt::CHAR;
    use self::winapi::shared::minwindef::DWORD;

    pub fn hostname() -> Result<String, ::std::io::Error> {
        let mut buffer = [0 as CHAR; 256];
        let mut len = buffer.len() as DWORD;

        unsafe {
            if GetComputerNameA(buffer.as_mut_ptr(), &mut len) == 0 {
                return Err(::std::io::Error::last_os_error())
            }
        }
        Ok(String::from_iter(buffer.iter().take(len as usize).map(|&i| i as u8 as char)))
    }

    pub fn username() -> Result<String, ::std::io::Error> {
        let mut buffer = [0 as CHAR; 256];
        let mut len = buffer.len() as DWORD;

        unsafe {
            if GetUserNameA(buffer.as_mut_ptr(), &mut len) == 0 {
                return Err(::std::io::Error::last_os_error())
            }
        }

        // Skip NUL byte
        if len > 0 {
            len -= 1;
        }

        Ok(String::from_iter(buffer.iter().take(len as usize).map(|&i| i as u8 as char)))
    }
}
