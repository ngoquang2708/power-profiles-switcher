use zbus::{Result, proxy};
use zvariant::{ObjectPath, OwnedObjectPath};

#[proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
pub trait UPower {
    fn enumerate_devices(&self) -> Result<Vec<OwnedObjectPath>>;

    fn get_critical_action(&self) -> Result<String>;

    fn get_display_device(&self) -> Result<OwnedObjectPath>;

    #[zbus(signal)]
    fn device_added(&self, device: ObjectPath<'_>) -> Result<()>;

    #[zbus(signal)]
    fn device_removed(&self, device: ObjectPath<'_>) -> Result<()>;

    #[zbus(property)]
    fn daemon_version(&self) -> Result<String>;

    #[zbus(property)]
    fn lid_is_closed(&self) -> Result<bool>;

    #[zbus(property)]
    fn lid_is_present(&self) -> Result<bool>;

    #[zbus(property)]
    fn on_battery(&self) -> Result<bool>;
}
