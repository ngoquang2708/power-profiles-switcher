use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use zbus::{Error, Result, proxy};
use zvariant::{OwnedValue, Type, Value};

#[proxy(
    interface = "net.hadess.PowerProfiles",
    default_service = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
pub trait PowerProfiles {
    #[zbus(name = "HoldProfile")]
    fn hold(&self, profile: &Profile, reason: &str, application_id: &str) -> Result<u32>;

    #[zbus(name = "ReleaseProfile")]
    fn release(&self, cookie: u32) -> Result<()>;

    #[zbus(signal, name = "ProfileReleased")]
    fn released(&self, cookie: u32) -> Result<()>;

    #[zbus(property)]
    fn actions(&self) -> Result<Vec<String>>;

    #[zbus(property, name = "ActiveProfile")]
    fn get_active(&self) -> Result<Profile>;

    #[zbus(property, name = "ActiveProfile")]
    fn set_active(&self, value: &Profile) -> Result<()>;

    #[zbus(property)]
    fn active_profile_holds(&self) -> Result<Vec<HashMap<String, OwnedValue>>>;

    #[zbus(property)]
    fn performance_degraded(&self) -> Result<String>;

    #[zbus(property)]
    fn performance_inhibited(&self) -> Result<String>;

    #[zbus(property)]
    fn profiles(&self) -> Result<Vec<HashMap<String, OwnedValue>>>;

    #[zbus(property)]
    fn version(&self) -> Result<String>;
}

#[derive(Debug, Copy, Clone, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s")]
pub enum Profile {
    PowerSaver,
    Balanced,
    Performance,
}

impl Profile {
    fn as_str(&self) -> &str {
        match self {
            Self::PowerSaver => "power-saver",
            Self::Balanced => "balanced",
            Self::Performance => "performance",
        }
    }
}

impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Profile {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "power-saver" => Ok(Self::PowerSaver),
            "balanced" => Ok(Self::Balanced),
            "performance" => Ok(Self::Performance),
            _ => Err(Self::Err::InvalidReply),
        }
    }
}

impl<'a> From<&'a Profile> for Value<'a> {
    fn from(value: &'a Profile) -> Self {
        Self::Str(value.as_str().into())
    }
}

impl TryFrom<OwnedValue> for Profile {
    type Error = Error;

    fn try_from(value: OwnedValue) -> Result<Self> {
        if let Value::Str(s) = &*value {
            if let Ok(p) = Profile::from_str(s) {
                return Ok(p);
            }
        }
        Err(Self::Error::InvalidReply)
    }
}
