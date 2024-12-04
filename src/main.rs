use std::time::{Duration, Instant};

use zbus::Connection;

use power_profiles_switcher::power_profiles::PowerProfilesProxy;
use power_profiles_switcher::sensors::{Matcher, SubFeatureFinder as _};

#[derive(Debug, Clone)]
struct Config {
    temp: f64,
    performance_profile_name: String,
}

#[derive(Debug, Copy, Clone)]
enum State {
    Normal,
    Prepare(Instant),
    Set(u32),
}

// TODO Disable on battery

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sensors = lm_sensors::Initializer::default().initialize()?;
    let matcher = Matcher {
        chip_name: "coretemp-isa-0000".to_string(),
        feat_name: "temp1".to_string(),
        feat_label: "Package id 0".to_string().into(),
        sub_feat_name: "temp1_input".to_string(),
    };
    let Some(sub_feat) = sensors.find(&matcher)? else {
        panic!("Sub-feature not found!");
    };
    let config = Config {
        temp: 65.0,
        performance_profile_name: "performance".to_string(),
    };
    let conn = Connection::system().await?;
    let proxy = PowerProfilesProxy::new(&conn).await?;
    let duration = Duration::from_secs(1);
    let mut state = State::Normal;
    while let Ok(temp) = sub_feat.value().map(|v| v.raw_value()) {
        let now = Instant::now();
        match (temp > config.temp, state) {
            (true, State::Normal) => {
                state = State::Prepare(now + Duration::from_secs(5));
                println!("temp={temp} {state:?}");
            }
            (true, State::Prepare(instant)) if now >= instant => {
                let cookie = proxy
                    .hold_profile(
                        &config.performance_profile_name,
                        "Temperature is rising",
                        "com.ngoquang2708.PowerProfilesSwitcher",
                    )
                    .await?;
                state = State::Set(cookie);
                println!("temp={temp} {state:?}");
            }
            (false, State::Prepare(instant)) if now >= instant => {
                state = State::Normal;
                println!("temp={temp} {state:?}");
            }
            (false, State::Set(cookie)) => {
                proxy.release_profile(cookie).await?;
                state = State::Normal;
                println!("temp={temp} {state:?}");
            }
            _ => {}
        }
        tokio::time::sleep(duration).await;
    }
    Ok(())
}
