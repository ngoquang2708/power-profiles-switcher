use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.UPower",
    default_service = "org.freedesktop.UPower",
    default_path = "/org/freedesktop/UPower"
)]
pub trait UPower {
    /// EnumerateDevices method
    fn enumerate_devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// GetCriticalAction method
    fn get_critical_action(&self) -> zbus::Result<String>;

    /// GetDisplayDevice method
    fn get_display_device(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// DeviceAdded signal
    #[zbus(signal)]
    fn device_added(&self, device: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// DeviceRemoved signal
    #[zbus(signal)]
    fn device_removed(&self, device: zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// DaemonVersion property
    #[zbus(property)]
    fn daemon_version(&self) -> zbus::Result<String>;

    /// LidIsClosed property
    #[zbus(property)]
    fn lid_is_closed(&self) -> zbus::Result<bool>;

    /// LidIsPresent property
    #[zbus(property)]
    fn lid_is_present(&self) -> zbus::Result<bool>;

    /// OnBattery property
    #[zbus(property)]
    fn on_battery(&self) -> zbus::Result<bool>;
}
