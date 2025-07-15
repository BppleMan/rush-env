use std::sync::LazyLock;
use serde::{Deserialize, Serialize};

pub static PLATFORM: LazyLock<Platform> = LazyLock::new(|| Platform::current());

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OS {
    macos,
    linux,
    #[default]
    unknown,
}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ARCH {
    x86_64,
    aarch64,
    #[default]
    unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Platform {
    pub os: OS,
    pub arch: ARCH,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformMatrix {
    pub os: Vec<OS>,
    pub arch: Vec<ARCH>,
}

impl Platform {
    pub fn current() -> Self {
        Self {
            os: match std::env::consts::OS {
                "linux" => OS::linux,
                "macos" => OS::macos,
                _ => OS::unknown,
            },
            arch: match std::env::consts::ARCH {
                "x86_64" => ARCH::x86_64,
                "aarch64" => ARCH::aarch64,
                _ => ARCH::unknown,
            },
        }
    }

    pub fn as_tag(&self) -> String {
        format!("{}-{}", self.os.as_str(), self.arch.as_str())
    }
}

impl OS {
    pub fn as_str(&self) -> &'static str {
        match self {
            OS::linux => "linux",
            OS::macos => "macos",
            OS::unknown => "unknown",
        }
    }
}

impl ARCH {
    pub fn as_str(&self) -> &'static str {
        match self {
            ARCH::x86_64 => "x86_64",
            ARCH::aarch64 => "aarch64",
            ARCH::unknown => "unknown",
        }
    }
}

impl Default for PlatformMatrix {
    fn default() -> Self {
        Self {
            os: vec![OS::linux, OS::macos],
            arch: vec![ARCH::x86_64, ARCH::aarch64],
        }
    }
}