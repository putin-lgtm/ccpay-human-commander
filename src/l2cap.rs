/// L2CAP raw socket layer.
/// Creates and connects Bluetooth L2CAP sockets using libc syscalls directly,
/// bypassing higher-level wrappers for maximum latency control.

use std::io;
#[cfg(target_os = "linux")]
use libc::{
    AF_BLUETOOTH, SOCK_SEQPACKET, close, connect, socket,
    sockaddr,
};
use libc::c_int as RawFd;

type SocklenT = u32;

/// BTPROTO_L2CAP protocol constant (not in libc stable yet).
pub const BTPROTO_L2CAP: libc::c_int = 0;

/// PSM 17 = HID Control channel.
pub const PSM_HID_CONTROL: u16 = 17;
/// PSM 19 = HID Interrupt channel (data path).
pub const PSM_HID_INTERRUPT: u16 = 19;

/// BlueZ sockaddr_l2 layout (matches kernel struct).
/// bdaddr is stored little-endian (byte-reversed from the canonical string).
#[cfg(target_os = "linux")]
#[repr(C)]
pub struct SockaddrL2 {
    pub l2_family: libc::c_ushort,
    pub l2_psm: u16,       // little-endian PSM
    pub l2_bdaddr: [u8; 6],
    pub l2_cid: u16,
    pub l2_bdaddr_type: u8,
}

/// Parse a "AA:BB:CC:DD:EE:FF" MAC string into the 6-byte little-endian
/// form expected by sockaddr_l2 (i.e., reversed byte order).
pub fn parse_bdaddr(mac: &str) -> io::Result<[u8; 6]> {
    let parts: Vec<&str> = mac.split(':').collect();
    if parts.len() != 6 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid MAC address"));
    }
    let mut addr = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        addr[5 - i] = u8::from_str_radix(p, 16)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid hex byte in MAC"))?;
    }
    Ok(addr)
}

/// Open a raw L2CAP SEQPACKET socket.
#[cfg(target_os = "linux")]
pub fn l2cap_socket() -> io::Result<RawFd> {
    // SAFETY: standard socket syscall; return value is checked.
    let fd = unsafe { socket(AF_BLUETOOTH, SOCK_SEQPACKET, BTPROTO_L2CAP) };
    if fd < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(fd)
}

/// Connect a raw L2CAP socket to the given remote `bdaddr` on `psm`.
/// `bdaddr` must already be in little-endian byte order (use `parse_bdaddr`).
#[cfg(target_os = "linux")]
pub fn l2cap_connect(fd: RawFd, bdaddr: &[u8; 6], psm: u16) -> io::Result<()> {
    let addr = SockaddrL2 {
        l2_family: AF_BLUETOOTH as libc::c_ushort,
        l2_psm: psm.to_le(),
        l2_bdaddr: *bdaddr,
        l2_cid: 0,
        l2_bdaddr_type: 0, // BDADDR_BREDR
    };

    let ret = unsafe {
        connect(
            fd,
            &addr as *const SockaddrL2 as *const sockaddr,
            std::mem::size_of::<SockaddrL2>() as SocklenT,
        )
    };

    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// Close an L2CAP socket file descriptor.
/// Silently ignores EBADF to allow clean teardown.
#[cfg(target_os = "linux")]
pub fn l2cap_close(fd: RawFd) {
    unsafe { close(fd) };
}

/// Wrapper holding both HID L2CAP channels for a single remote device.
#[cfg(target_os = "linux")]
pub struct HidChannels {
    pub control_fd: RawFd,
    pub interrupt_fd: RawFd,
}

#[cfg(target_os = "linux")]
impl HidChannels {
    /// Open and connect both HID Control (PSM 17) and HID Interrupt (PSM 19)
    /// sockets to the remote device identified by `mac`.
    pub fn connect(mac: &str) -> io::Result<Self> {
        let bdaddr = parse_bdaddr(mac)?;

        let control_fd = l2cap_socket()?;
        if let Err(e) = l2cap_connect(control_fd, &bdaddr, PSM_HID_CONTROL) {
            l2cap_close(control_fd);
            return Err(e);
        }

        let interrupt_fd = l2cap_socket()?;
        if let Err(e) = l2cap_connect(interrupt_fd, &bdaddr, PSM_HID_INTERRUPT) {
            l2cap_close(control_fd);
            l2cap_close(interrupt_fd);
            return Err(e);
        }

        Ok(Self { control_fd, interrupt_fd })
    }
}

#[cfg(target_os = "linux")]
impl Drop for HidChannels {
    fn drop(&mut self) {
        l2cap_close(self.interrupt_fd);
        l2cap_close(self.control_fd);
    }
}
