use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

pub static PLATFORM: LazyLock<Platform> = LazyLock::new(Platform::current);

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OS {
    macos,
    linux,
    #[default]
    unknown,
}

#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ARCH {
    x86_64,
    aarch64,
    #[default]
    unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Platform {
    #[serde(rename = "@os", default)]
    pub os: Option<OS>,
    #[serde(rename = "@arch", default)]
    pub arch: Option<ARCH>,
}

impl Platform {
    pub fn current() -> Self {
        Self {
            os: match std::env::consts::OS {
                "linux" => Some(OS::linux),
                "macos" => Some(OS::macos),
                _ => Some(OS::unknown),
            },
            arch: match std::env::consts::ARCH {
                "x86_64" => Some(ARCH::x86_64),
                "aarch64" => Some(ARCH::aarch64),
                _ => Some(ARCH::unknown),
            },
        }
    }

    pub fn as_tag(&self) -> String {
        let mut tag = String::new();
        if let Some(os) = &self.os {
            tag.push_str(os.as_str());
        } else {
            tag.push_str("unknown");
        }
        if let Some(arch) = &self.arch {
            tag.push('-');
            tag.push_str(arch.as_str());
        }
        tag
    }

    pub fn contains_current(&self) -> bool {
        let current = &PLATFORM;
        (self.os.is_none() || self.os == current.os) && (self.arch.is_none() || self.arch == current.arch)
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
