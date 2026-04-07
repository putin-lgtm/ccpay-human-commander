/// BlueZ D-Bus profile registration via zbus.
///
/// Registers a HID profile (UUID 0x1124) with BlueZ's ProfileManager1
/// so that BlueZ exposes the keyboard SDP record and routes incoming
/// HID connections to this process.

use std::collections::HashMap;
use zbus::{Connection, proxy, zvariant::Value, zvariant::OwnedObjectPath};
use crate::sdp::HID_SDP_RECORD;

/// HID Profile UUID (Bluetooth assigned number for Human Interface Device).
pub const HID_PROFILE_UUID: &str = "00001124-0000-1000-8000-00805f9b34fb";

/// D-Bus object path where we expose the Profile1 implementation.
pub const PROFILE_OBJECT_PATH: &str = "/com/ccpay/hid/keyboard";

/// Proxy for org.bluez.ProfileManager1 on the system bus.
#[proxy(
    interface = "org.bluez.ProfileManager1",
    default_service = "org.bluez",
    default_path = "/org/bluez"
)]
trait ProfileManager1 {
    /// Register a profile object at `profile` with `uuid` and `options`.
    fn register_profile(
        &self,
        profile: OwnedObjectPath,
        uuid: &str,
        options: HashMap<String, Value<'_>>,
    ) -> zbus::Result<()>;

    /// Unregister a previously registered profile.
    fn unregister_profile(&self, profile: OwnedObjectPath) -> zbus::Result<()>;
}

/// Build the options map for RegisterProfile.
/// Includes the full SDP XML record so BlueZ exposes a proper keyboard entry.
fn build_profile_options() -> HashMap<String, Value<'static>> {
    let mut opts: HashMap<String, Value<'static>> = HashMap::new();

    // ServiceRecord: raw SDP XML — BlueZ will parse and register it.
    opts.insert(
        "ServiceRecord".to_owned(),
        Value::from(HID_SDP_RECORD),
    );

    // Role: "server" means we accept connections (act as the keyboard).
    opts.insert("Role".to_owned(), Value::from("server"));

    // RequireAuthentication: keyboards typically do not require PIN pairing.
    opts.insert("RequireAuthentication".to_owned(), Value::from(false));

    // RequireAuthorization: allow connection without explicit user approval.
    opts.insert("RequireAuthorization".to_owned(), Value::from(false));

    // AutoConnect: reconnect automatically when the remote device is in range.
    opts.insert("AutoConnect".to_owned(), Value::from(true));

    opts
}

/// Connect to the system D-Bus and register the HID keyboard profile with BlueZ.
///
/// Returns the open `Connection` so the caller can keep the bus alive
/// (BlueZ will unregister the profile if the connection drops).
pub async fn register_hid_profile() -> zbus::Result<Connection> {
    let conn = Connection::system().await?;

    let manager = ProfileManager1Proxy::new(&conn).await?;

    let profile_path = OwnedObjectPath::try_from(PROFILE_OBJECT_PATH)
        .expect("static profile path is always valid");

    let options = build_profile_options();

    manager
        .register_profile(profile_path, HID_PROFILE_UUID, options)
        .await?;

    println!("[dbus] HID profile registered: {HID_PROFILE_UUID}");
    Ok(conn)
}

/// Unregister the HID profile. Called on clean shutdown.
pub async fn unregister_hid_profile(conn: &Connection) -> zbus::Result<()> {
    let manager = ProfileManager1Proxy::new(conn).await?;
    let profile_path = OwnedObjectPath::try_from(PROFILE_OBJECT_PATH)
        .expect("static profile path is always valid");
    manager.unregister_profile(profile_path).await?;
    println!("[dbus] HID profile unregistered");
    Ok(())
}
