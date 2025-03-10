use std::str::FromStr;

use serde::{Deserialize, Serialize};
use zbus::{Error, Result, proxy};
use zvariant::{OwnedObjectPath, OwnedValue, Type, Value};

#[proxy(
    interface = "org.freedesktop.login1.User",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1/user/self"
)]
pub trait User {
    fn kill(&self, signal_number: i32) -> Result<()>;

    fn terminate(&self) -> Result<()>;

    #[zbus(property)]
    fn display(&self) -> Result<(String, OwnedObjectPath)>;

    #[zbus(property)]
    fn idle_hint(&self) -> Result<bool>;

    #[zbus(property)]
    fn idle_since_hint(&self) -> Result<u64>;

    #[zbus(property)]
    fn idle_since_hint_monotonic(&self) -> Result<u64>;

    #[zbus(property)]
    fn linger(&self) -> Result<bool>;

    #[zbus(property)]
    fn name(&self) -> Result<String>;

    #[zbus(property)]
    fn runtime_path(&self) -> Result<String>;

    #[zbus(property)]
    fn service(&self) -> Result<String>;

    #[zbus(property)]
    fn sessions(&self) -> Result<Vec<(String, OwnedObjectPath)>>;

    #[zbus(property)]
    fn slice(&self) -> Result<String>;

    #[zbus(property)]
    fn state(&self) -> Result<State>;

    #[zbus(property)]
    fn timestamp(&self) -> Result<u64>;

    #[zbus(property)]
    fn timestamp_monotonic(&self) -> Result<u64>;

    #[zbus(property, name = "UID")]
    fn uid(&self) -> Result<u32>;

    #[zbus(property, name = "GID")]
    fn gid(&self) -> Result<u32>;
}

#[derive(Debug, Clone, Copy, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    Offline,
    Lingering,
    Online,
    Active,
    Closing,
}

impl FromStr for State {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "offline" => Ok(Self::Offline),
            "lingering" => Ok(Self::Lingering),
            "online" => Ok(Self::Online),
            "active" => Ok(Self::Active),
            "closing" => Ok(Self::Lingering),
            _ => Err(Self::Err::InvalidReply),
        }
    }
}

impl TryFrom<OwnedValue> for State {
    type Error = Error;

    fn try_from(value: OwnedValue) -> Result<Self> {
        if let Value::Str(s) = &*value {
            if let Ok(s) = State::from_str(s) {
                return Ok(s);
            }
        }
        Err(Self::Error::InvalidReply)
    }
}
