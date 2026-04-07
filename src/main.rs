#[cfg(target_os = "linux")]
mod cli;
#[cfg(target_os = "linux")]
mod hid;
#[cfg(target_os = "linux")]
mod l2cap;
#[cfg(target_os = "linux")]
mod profile;
mod sdp;  // sdp is platform-neutral (pure string constant)

/// Target Android device MAC address.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const TARGET_MAC: &str = "F4:7D:EF:8A:3B:5C";

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("ccpay-human-commander only runs on Linux (requires BlueZ / L2CAP).");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
#[tokio::main]
async fn main() {
    println!("ccpay-human-commander — Bluetooth HID Keyboard Emulator");
    println!("Target device: {TARGET_MAC}");

    // -----------------------------------------------------------------------
    // Step 1: Register the HID profile with BlueZ via D-Bus.
    // This populates the SDP record so the Android device sees us as a
    // "Standard Bluetooth Keyboard" during service discovery.
    // -----------------------------------------------------------------------
    let dbus_conn = match profile::register_hid_profile().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[main] Failed to register HID profile: {e}");
            eprintln!("[main] Ensure BlueZ is running (bluetoothd) and you have sufficient permissions.");
            std::process::exit(1);
        }
    };

    // -----------------------------------------------------------------------
    // Step 2: Open raw L2CAP sockets and connect both HID channels.
    // PSM 17 = HID Control | PSM 19 = HID Interrupt
    // -----------------------------------------------------------------------
    println!("[main] Connecting L2CAP channels to {TARGET_MAC} …");
    let channels = match l2cap::HidChannels::connect(TARGET_MAC) {
        Ok(ch) => ch,
        Err(e) => {
            eprintln!("[main] L2CAP connect failed: {e}");
            eprintln!("[main] Make sure the device is paired, reachable, and BlueZ is up.");
            // Attempt clean profile unregister before exit.
            let _ = profile::unregister_hid_profile(&dbus_conn).await;
            std::process::exit(1);
        }
    };
    println!("[main] HID channels connected (ctrl_fd={}, intr_fd={})",
        channels.control_fd, channels.interrupt_fd);

    // -----------------------------------------------------------------------
    // Step 3: Run the interactive CLI.
    // Blocks until the user types 'quit' or closes stdin.
    // -----------------------------------------------------------------------
    cli::run_cli(channels.interrupt_fd);

    // -----------------------------------------------------------------------
    // Step 4: Clean shutdown — unregister profile and drop sockets.
    // -----------------------------------------------------------------------
    let _ = profile::unregister_hid_profile(&dbus_conn).await;
    // `channels` is dropped here, closing both L2CAP fds automatically.
    println!("[main] Shutdown complete.");
}
