use std::time::{Duration, Instant};
use std::{fmt, fs};

use anyhow::{anyhow, Context};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use zbus::Connection;

use power_profiles_switcher::power_profiles::PowerProfilesProxy;
use power_profiles_switcher::sensors::{Matcher, SubFeatureFinder as _};
use power_profiles_switcher::upower::UPowerProxy;

static APP_ID: &str = "com.ngoquang2708.PowerProfilesSwitcher";
static REASON: &str = "Temperature is rising";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    matcher: Matcher,
    temp: f64,
    profile: PowerProfiles,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PowerProfiles {
    PowerSaver,
    Balanced,
    Performance,
}

impl fmt::Display for PowerProfiles {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PowerSaver => write!(f, "power-saver"),
            Self::Balanced => write!(f, "balanced"),
            Self::Performance => write!(f, "performance"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum State {
    Normal,
    Prepare(Instant),
    Set(u32),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    let sensors = lm_sensors::Initializer::default().initialize()?;
    let sub_feat = sensors
        .find(&config.matcher)
        .context("finding sub-feature")?
        .ok_or(anyhow!("Sub-feature not found!"))?;
    let conn = Connection::system().await?;
    let power_profiles_proxy = PowerProfilesProxy::new(&conn).await?;
    let upower_proxy = UPowerProxy::new(&conn).await?;
    let duration = Duration::from_secs(1);
    let mut state = State::Normal;
    while let Ok(temp) = sub_feat.value().map(|v| v.raw_value()) {
        tokio::time::sleep(duration).await;
        if upower_proxy.on_battery().await? {
            if let State::Set(cookie) = state {
                let _ = power_profiles_proxy.release_profile(cookie).await;
            }
            state = State::Normal;
            continue;
        }
        let now = Instant::now();
        match (temp > config.temp, state) {
            (true, State::Normal) => {
                state = State::Prepare(now + Duration::from_secs(5));
                println!("temp={temp} {state:?}");
            }
            (true, State::Prepare(instant)) if now >= instant => {
                let cookie = power_profiles_proxy
                    .hold_profile(&config.profile.to_string(), REASON, APP_ID)
                    .await?;
                state = State::Set(cookie);
                println!("temp={temp} {state:?}");
            }
            (false, State::Prepare(instant)) if now >= instant => {
                state = State::Normal;
                println!("temp={temp} {state:?}");
            }
            (false, State::Set(cookie)) => {
                if power_profiles_proxy.release_profile(cookie).await.is_err() {
                    println!("Power profile is changed by other programs!");
                }
                state = State::Normal;
                println!("temp={temp} {state:?}");
            }
            _ => {}
        }
    }
    Ok(())
}

fn load_config() -> anyhow::Result<Config> {
    let dir = BaseDirs::new()
        .context("detect user's directories")?
        .config_dir()
        .join("PowerProfilesSwitcher");
    let path = dir.join("config.toml");
    let text = fs::read_to_string(path).context("reading config.toml")?;
    toml::from_str(&text).context("deserializing config content")
}
