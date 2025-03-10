use std::fs;
use std::time::{Duration, Instant};

use anyhow::{Context, anyhow};
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use zbus::Connection;

use power_profiles_switcher::login1_user::{State as UserState, UserProxy};
use power_profiles_switcher::power_profiles::{PowerProfilesProxy, Profile};
use power_profiles_switcher::sensors::{Matcher, SubFeatureFinder as _};
use power_profiles_switcher::upower::UPowerProxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Config {
    matcher: Matcher,
    temp: f64,
    inactive_profile: Profile,
    active_profile: Profile,
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
    let pp_proxy = PowerProfilesProxy::new(&conn).await?;
    let upower_proxy = UPowerProxy::new(&conn).await?;
    let user_proxy = UserProxy::new(&conn).await?;
    let duration = Duration::from_secs(1);
    let mut state = State::Inactive;
    let set_profile = async |profile| {
        if let Err(err) = pp_proxy.set_active(profile).await {
            // Ignore errors if user is inactive
            if !matches!(user_proxy.state().await?, UserState::Active) {
                return Err(err);
            }
        }
        Ok(())
    };
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
                println!("Set power profile to {}!", config.inactive_profile);
                set_profile(&config.inactive_profile).await?;
            }
            println!("Exiting...");
            break;
        }
        if upower_proxy.on_battery().await? {
            if let State::Active = state {
                println!("Set power profile to {}!", config.inactive_profile);
                set_profile(&config.inactive_profile).await?;
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
                set_profile(&config.active_profile).await?;
                state = State::Active;
                println!("temp={temp} {state:?}");
            }
            (false, State::Prepare(instant)) if now >= instant => {
                state = State::Inactive;
                println!("temp={temp} {state:?}");
            }
            (false, State::Active) => {
                set_profile(&config.inactive_profile).await?;
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
