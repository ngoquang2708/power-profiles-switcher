use std::time::{Duration, Instant};
use std::{fmt, fs};

use anyhow::{anyhow, Context};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

use power_profiles_switcher::power_profiles::PowerProfilesProxy;
use power_profiles_switcher::sensors::{Matcher, SubFeatureFinder as _};
use power_profiles_switcher::upower::UPowerProxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    matcher: Matcher,
    temp: f64,
    inactive_profile: PowerProfiles,
    active_profile: PowerProfiles,
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
    Inactive,
    Prepare(Instant),
    Active,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config()?;
    let shutdown_token = CancellationToken::new();
    let sensors = lm_sensors::Initializer::default().initialize()?;
    let sub_feat = sensors
        .find(&config.matcher)
        .context("finding sub-feature")?
        .ok_or(anyhow!("Sub-feature not found!"))?;
    let conn = Connection::system().await?;
    let power_profiles_proxy = PowerProfilesProxy::new(&conn).await?;
    let upower_proxy = UPowerProxy::new(&conn).await?;
    let duration = Duration::from_secs(1);
    let mut state = State::Inactive;
    tokio::spawn({
        let shutdown_token = shutdown_token.clone();
        async move {
            use tokio::signal::unix;
            let Ok(mut terminate) = unix::signal(unix::SignalKind::terminate()) else {
                println!("Failed to initialize terminate signal!");
                return;
            };
            let interrupt = tokio::signal::ctrl_c();
            tokio::select! {
                _ = terminate.recv() => {
                    println!("Received Terminate signal.");

                }
                _ = interrupt => {
                    println!("Received Interrupt signal.");
                }
            }
            shutdown_token.cancel();
        }
    });
    while let Ok(temp) = sub_feat.value().map(|v| v.raw_value()) {
        tokio::time::sleep(duration).await;
        if shutdown_token.is_cancelled() {
            if let State::Active = state {
                let profile = config.inactive_profile.to_string();
                println!("Set power profile to {profile}!");
                power_profiles_proxy
                    .set_active_profile(profile.as_str())
                    .await?;
            }
            println!("Exiting...");
            break;
        }
        if upower_proxy.on_battery().await? {
            if let State::Active = state {
                let profile = config.inactive_profile.to_string();
                println!("Set power profile to {profile}!");
                power_profiles_proxy
                    .set_active_profile(profile.as_str())
                    .await?;
            }
            state = State::Inactive;
            continue;
        }
        let now = Instant::now();
        match (temp > config.temp, state) {
            (true, State::Inactive) => {
                state = State::Prepare(now + Duration::from_secs(5));
                println!("temp={temp} {state:?}");
            }
            (true, State::Prepare(instant)) if now >= instant => {
                power_profiles_proxy
                    .set_active_profile(config.active_profile.to_string().as_str())
                    .await?;
                state = State::Active;
                println!("temp={temp} {state:?}");
            }
            (false, State::Prepare(instant)) if now >= instant => {
                state = State::Inactive;
                println!("temp={temp} {state:?}");
            }
            (false, State::Active) => {
                power_profiles_proxy
                    .set_active_profile(config.inactive_profile.to_string().as_str())
                    .await?;
                state = State::Inactive;
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
