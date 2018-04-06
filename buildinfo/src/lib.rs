extern crate rustc_version;

use std::env;
use std::fmt::Display;
use std::str::FromStr;

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
        "TARGET", target, &'static str,
        var("TARGET"),
        |x| x,
    );
    (
        "HOST", host, &'static str,
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
);

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
