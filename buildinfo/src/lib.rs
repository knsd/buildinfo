extern crate rustc_version;

use std::env;
use std::fmt::Display;
use std::str::FromStr;
use std::time::{UNIX_EPOCH, SystemTime, SystemTimeError};

pub use rustc_version::Version;

macro_rules! __make {
    ($(($varname:expr, $fieldname:ident, $field_ty:ty, $prepare:expr, $extract:expr,));*; ) => {

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
            $fieldname: &'static str,
        )*
    ) -> Self {
        $(
            let $fieldname: $field_ty = $extract($fieldname);
        )*

        Self {
            $(
                $fieldname,
            )*
        }
    }
}

pub fn prepare() {
    $(
        print_env($varname, $prepare);
    )*
}

#[macro_export]
macro_rules! build_info {
    () => {
        $crate::BuildInfo::new(
            $(
                __build_info_var!($varname),
            )*
        )
    };
}

    }
}

__make!(
    (
        "TARGET_TRIPLE", target_triple, &'static str,
        var("TARGET"),
        |x| x,
    );
    (
        "HOST_TRIPLE", host_triple, &'static str,
        var("HOST"),
        |x| x,
    );
    (
        "OPT_LEVEL", opt_level, &'static str,
        var("OPT_LEVEL"),
        |x| x,

    );
    (
        "DEBUG", debug, bool,
        var("DEBUG"),
        |x| bool::from_str(x).expect("buildinfo debug"),
    );
    (
        "PROFILE", profile, &'static str,
        var("PROFILE"),
        |x| x,
    );

    (
        "RUSTC_VERSION", rustc_version, Version,
        rustc_version::version().expect("buildinfo prepare rustc_version"),
        |x| Version::parse(x).expect("buildinfo build rustc_version"),
    );
    (
        "COMPILED_AT", compiled_at, u64,
        now().expect("buildinfo prepare now"),
        |x| u64::from_str(x).expect("buildinfo build now"),
    );

);

impl BuildInfo {
    pub fn target_triple(&self) -> &str {
        self.target_triple
    }

    pub fn host_triple(&self) -> &str {
        self.host_triple
    }

    pub fn opt_level(&self) -> &str {
        self.opt_level
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn profile(&self) -> &str {
        self.profile
    }

    pub fn rustc_version(&self) -> &Version {
        &self.rustc_version
    }

    pub fn compiled_at(&self) -> u64 {
        self.compiled_at
    }
}

fn print_env<K: Display, V: Display>(key: K, value: V) {
    println!("cargo:rustc-env=BUILD_INFO_{}={}", key, value)
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
macro_rules! __build_info_var {
    ($name:expr) => {
        env!(concat!("BUILD_INFO_", $name))
    };
}

fn now() -> Result<u64, SystemTimeError> {
    let now = SystemTime::now();
    let elapsed = now.duration_since(UNIX_EPOCH)?;
    Ok(elapsed.as_secs())
}
