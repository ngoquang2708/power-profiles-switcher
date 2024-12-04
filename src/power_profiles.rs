use std::collections::HashMap;

use zbus::proxy;

#[proxy(
    interface = "net.hadess.PowerProfiles",
    default_service = "net.hadess.PowerProfiles",
    default_path = "/net/hadess/PowerProfiles"
)]
pub trait PowerProfiles {
    /// HoldProfile method
    fn hold_profile(&self, profile: &str, reason: &str, application_id: &str) -> zbus::Result<u32>;

    /// ReleaseProfile method
    fn release_profile(&self, cookie: u32) -> zbus::Result<()>;

    /// ProfileReleased signal
    #[zbus(signal)]
    fn profile_released(&self, cookie: u32) -> zbus::Result<()>;

    /// Actions property
    #[zbus(property)]
    fn actions(&self) -> zbus::Result<Vec<String>>;

    /// ActiveProfile property
    #[zbus(property)]
    fn active_profile(&self) -> zbus::Result<String>;
    #[zbus(property)]
    fn set_active_profile(&self, value: &str) -> zbus::Result<()>;

    /// ActiveProfileHolds property
    #[zbus(property)]
    fn active_profile_holds(
        &self,
    ) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// PerformanceDegraded property
    #[zbus(property)]
    fn performance_degraded(&self) -> zbus::Result<String>;

    /// PerformanceInhibited property
    #[zbus(property)]
    fn performance_inhibited(&self) -> zbus::Result<String>;

    /// Profiles property
    #[zbus(property)]
    fn profiles(&self) -> zbus::Result<Vec<HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// Version property
    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}
