use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use zbus::proxy;
use zvariant::{OwnedValue, Type, Value};

#[proxy(
    interface = "net.hadess.PowerProfiles",
    default_service = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
pub trait PowerProfiles {
    #[zbus(name = "HoldProfile")]
    fn hold(&self, profile: &Profiles, reason: &str, application_id: &str) -> zbus::Result<u32>;

    #[zbus(name = "ReleaseProfile")]
    fn release(&self, cookie: u32) -> zbus::Result<()>;

    #[zbus(signal, name = "ProfileReleased")]
    fn released(&self, cookie: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn actions(&self) -> zbus::Result<Vec<String>>;

    #[zbus(property, name = "ActiveProfile")]
    fn get_active(&self) -> zbus::Result<Profiles>;

    #[zbus(property, name = "ActiveProfile")]
    fn set_active(&self, value: &Profiles) -> zbus::Result<()>;

    #[zbus(property)]
    fn active_profile_holds(
        &self,
    ) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::OwnedValue>>>;

    #[zbus(property)]
    fn performance_degraded(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn performance_inhibited(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn profiles(&self) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::OwnedValue>>>;

    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}

#[derive(Debug, Copy, Clone, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
#[zvariant(signature = "s")]
pub enum Profiles {
    PowerSaver,
    Balanced,
    Performance,
}

impl Profiles {
    fn as_str(&self) -> &str {
        match self {
            Self::PowerSaver => "power-saver",
            Self::Balanced => "balanced",
            Self::Performance => "performance",
        }
    }
}

impl fmt::Display for Profiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Profiles {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "power-saver" => Ok(Self::PowerSaver),
            "balanced" => Ok(Self::Balanced),
            "performance" => Ok(Self::Performance),
            _ => Err(()),
        }
    }
}

impl<'a> From<&'a Profiles> for Value<'a> {
    fn from(value: &'a Profiles) -> Self {
        Self::Str(value.as_str().into())
    }
}

impl TryFrom<OwnedValue> for Profiles {
    type Error = zbus::Error;

    fn try_from(value: OwnedValue) -> zbus::Result<Self> {
        if let Value::Str(s) = &*value {
            if let Ok(p) = Profiles::from_str(s) {
                return Ok(p);
            }
        }
        Err(Self::Error::InvalidReply)
    }
}
