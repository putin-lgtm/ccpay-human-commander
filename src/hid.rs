/// HID Keyboard Report module.
///
/// Formats and sends 8-byte Keyboard Input Reports over an L2CAP interrupt
/// socket using the standard boot-keyboard report format:
///
///   Byte 0: 0xA1  – HID Input report identifier
///   Byte 1: modifier bitmask
///   Byte 2: reserved (0x00)
///   Byte 3: keycode 1
///   Bytes 4-7: keycodes 2-5 (unused, 0x00)
///
/// USB HID Usage Table keycodes are defined in the `KeyCode` enum below.

use std::io;
#[cfg(target_os = "linux")]
use libc::{c_int as RawFd, send, MSG_NOSIGNAL};

/// On Linux, RawFd is libc::c_int. This re-export keeps the rest of the
/// module compiling on any host.
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
type RawFd = libc::c_int;

// ---------------------------------------------------------------------------
// USB HID Usage Table: Keyboard/Keypad page (0x07)
// ---------------------------------------------------------------------------

/// Modifier bitmask flags (byte 1 of the report).
#[allow(dead_code)]
pub mod modifier {
    pub const NONE: u8 = 0x00;
    pub const LEFT_CTRL: u8 = 0x01;
    pub const LEFT_SHIFT: u8 = 0x02;
    pub const LEFT_ALT: u8 = 0x04;
    pub const LEFT_GUI: u8 = 0x08;
    pub const RIGHT_CTRL: u8 = 0x10;
    pub const RIGHT_SHIFT: u8 = 0x20;
    pub const RIGHT_ALT: u8 = 0x40;
    pub const RIGHT_GUI: u8 = 0x80;
}

/// Common USB HID keycodes (Usage ID, Keyboard/Keypad page).
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCode {
    // Control keys
    None = 0x00,
    Enter = 0x28,
    Escape = 0x29,
    Backspace = 0x2A,
    Tab = 0x2B,
    Space = 0x2C,
    CapsLock = 0x39,

    // Navigation
    PageUp = 0x4B,
    PageDown = 0x4E,
    Home = 0x4A,
    End = 0x4D,
    ArrowRight = 0x4F,
    ArrowLeft = 0x50,
    ArrowDown = 0x51,
    ArrowUp = 0x52,

    // Alphanumeric — letters (lowercase; use LEFT_SHIFT modifier for uppercase)
    A = 0x04,
    B = 0x05,
    C = 0x06,
    D = 0x07,
    E = 0x08,
    F = 0x09,
    G = 0x0A,
    H = 0x0B,
    I = 0x0C,
    J = 0x0D,
    K = 0x0E,
    L = 0x0F,
    M = 0x10,
    N = 0x11,
    O = 0x12,
    P = 0x13,
    Q = 0x14,
    R = 0x15,
    S = 0x16,
    T = 0x17,
    U = 0x18,
    V = 0x19,
    W = 0x1A,
    X = 0x1B,
    Y = 0x1C,
    Z = 0x1D,

    // Digits (top row)
    Digit1 = 0x1E,
    Digit2 = 0x1F,
    Digit3 = 0x20,
    Digit4 = 0x21,
    Digit5 = 0x22,
    Digit6 = 0x23,
    Digit7 = 0x24,
    Digit8 = 0x25,
    Digit9 = 0x26,
    Digit0 = 0x27,

    // Function keys
    F1 = 0x3A,
    F2 = 0x3B,
    F3 = 0x3C,
    F4 = 0x3D,
    F5 = 0x3E,
    F6 = 0x3F,
    F7 = 0x40,
    F8 = 0x41,
    F9 = 0x42,
    F10 = 0x43,
    F11 = 0x44,
    F12 = 0x45,
}

// ---------------------------------------------------------------------------
// Report construction
// ---------------------------------------------------------------------------

/// Build a standard 8-byte HID keyboard input report.
///
/// # Arguments
/// * `modifier` – bitmask from the `modifier` module (or 0x00 for none).
/// * `keycode`  – USB HID usage ID of the key being pressed (0x00 = none).
#[inline]
pub fn build_report(modifier: u8, keycode: u8) -> [u8; 8] {
    [0xA1, modifier, 0x00, keycode, 0x00, 0x00, 0x00, 0x00]
}

/// Build the key-release report (all fields zeroed, except the 0xA1 header).
#[inline]
pub fn build_release_report() -> [u8; 8] {
    [0xA1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
}

// ---------------------------------------------------------------------------
// Low-level send over L2CAP fd
// ---------------------------------------------------------------------------

/// Send `data` over the given L2CAP file descriptor using `libc::send`.
/// `MSG_NOSIGNAL` suppresses SIGPIPE on broken connections.
#[cfg(target_os = "linux")]
fn send_report(fd: RawFd, data: &[u8]) -> io::Result<()> {
    let ret = unsafe {
        send(
            fd,
            data.as_ptr() as *const libc::c_void,
            data.len(),
            MSG_NOSIGNAL,
        )
    };
    if ret < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
fn send_report(_fd: RawFd, _data: &[u8]) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Linux only"))
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Send a key-press report for `keycode` with optional `modifier` bits
/// over the HID Interrupt L2CAP channel (`interrupt_fd`).
pub fn key_press(interrupt_fd: RawFd, modifier: u8, keycode: KeyCode) -> io::Result<()> {
    let report = build_report(modifier, keycode as u8);
    send_report(interrupt_fd, &report)
}

/// Send a key-release report (all keys up) over `interrupt_fd`.
pub fn key_release(interrupt_fd: RawFd) -> io::Result<()> {
    let report = build_release_report();
    send_report(interrupt_fd, &report)
}

/// Type a single character by sending a press followed immediately by a release.
/// Handles uppercase letters automatically by asserting LEFT_SHIFT.
pub fn type_char(interrupt_fd: RawFd, ch: char) -> io::Result<()> {
    if let Some((kc, mods)) = char_to_keycode(ch) {
        key_press(interrupt_fd, mods, kc)?;
        key_release(interrupt_fd)?;
    }
    Ok(())
}

/// Type a string one character at a time.
pub fn type_string(interrupt_fd: RawFd, s: &str) -> io::Result<()> {
    for ch in s.chars() {
        type_char(interrupt_fd, ch)?;
    }
    Ok(())
}

/// Map an ASCII character to its (KeyCode, modifier) pair.
/// Returns `None` for unmapped characters.
fn char_to_keycode(ch: char) -> Option<(KeyCode, u8)> {
    match ch {
        'a' => Some((KeyCode::A, modifier::NONE)),
        'b' => Some((KeyCode::B, modifier::NONE)),
        'c' => Some((KeyCode::C, modifier::NONE)),
        'd' => Some((KeyCode::D, modifier::NONE)),
        'e' => Some((KeyCode::E, modifier::NONE)),
        'f' => Some((KeyCode::F, modifier::NONE)),
        'g' => Some((KeyCode::G, modifier::NONE)),
        'h' => Some((KeyCode::H, modifier::NONE)),
        'i' => Some((KeyCode::I, modifier::NONE)),
        'j' => Some((KeyCode::J, modifier::NONE)),
        'k' => Some((KeyCode::K, modifier::NONE)),
        'l' => Some((KeyCode::L, modifier::NONE)),
        'm' => Some((KeyCode::M, modifier::NONE)),
        'n' => Some((KeyCode::N, modifier::NONE)),
        'o' => Some((KeyCode::O, modifier::NONE)),
        'p' => Some((KeyCode::P, modifier::NONE)),
        'q' => Some((KeyCode::Q, modifier::NONE)),
        'r' => Some((KeyCode::R, modifier::NONE)),
        's' => Some((KeyCode::S, modifier::NONE)),
        't' => Some((KeyCode::T, modifier::NONE)),
        'u' => Some((KeyCode::U, modifier::NONE)),
        'v' => Some((KeyCode::V, modifier::NONE)),
        'w' => Some((KeyCode::W, modifier::NONE)),
        'x' => Some((KeyCode::X, modifier::NONE)),
        'y' => Some((KeyCode::Y, modifier::NONE)),
        'z' => Some((KeyCode::Z, modifier::NONE)),
        'A' => Some((KeyCode::A, modifier::LEFT_SHIFT)),
        'B' => Some((KeyCode::B, modifier::LEFT_SHIFT)),
        'C' => Some((KeyCode::C, modifier::LEFT_SHIFT)),
        'D' => Some((KeyCode::D, modifier::LEFT_SHIFT)),
        'E' => Some((KeyCode::E, modifier::LEFT_SHIFT)),
        'F' => Some((KeyCode::F, modifier::LEFT_SHIFT)),
        'G' => Some((KeyCode::G, modifier::LEFT_SHIFT)),
        'H' => Some((KeyCode::H, modifier::LEFT_SHIFT)),
        'I' => Some((KeyCode::I, modifier::LEFT_SHIFT)),
        'J' => Some((KeyCode::J, modifier::LEFT_SHIFT)),
        'K' => Some((KeyCode::K, modifier::LEFT_SHIFT)),
        'L' => Some((KeyCode::L, modifier::LEFT_SHIFT)),
        'M' => Some((KeyCode::M, modifier::LEFT_SHIFT)),
        'N' => Some((KeyCode::N, modifier::LEFT_SHIFT)),
        'O' => Some((KeyCode::O, modifier::LEFT_SHIFT)),
        'P' => Some((KeyCode::P, modifier::LEFT_SHIFT)),
        'Q' => Some((KeyCode::Q, modifier::LEFT_SHIFT)),
        'R' => Some((KeyCode::R, modifier::LEFT_SHIFT)),
        'S' => Some((KeyCode::S, modifier::LEFT_SHIFT)),
        'T' => Some((KeyCode::T, modifier::LEFT_SHIFT)),
        'U' => Some((KeyCode::U, modifier::LEFT_SHIFT)),
        'V' => Some((KeyCode::V, modifier::LEFT_SHIFT)),
        'W' => Some((KeyCode::W, modifier::LEFT_SHIFT)),
        'X' => Some((KeyCode::X, modifier::LEFT_SHIFT)),
        'Y' => Some((KeyCode::Y, modifier::LEFT_SHIFT)),
        'Z' => Some((KeyCode::Z, modifier::LEFT_SHIFT)),
        '0' => Some((KeyCode::Digit0, modifier::NONE)),
        '1' => Some((KeyCode::Digit1, modifier::NONE)),
        '2' => Some((KeyCode::Digit2, modifier::NONE)),
        '3' => Some((KeyCode::Digit3, modifier::NONE)),
        '4' => Some((KeyCode::Digit4, modifier::NONE)),
        '5' => Some((KeyCode::Digit5, modifier::NONE)),
        '6' => Some((KeyCode::Digit6, modifier::NONE)),
        '7' => Some((KeyCode::Digit7, modifier::NONE)),
        '8' => Some((KeyCode::Digit8, modifier::NONE)),
        '9' => Some((KeyCode::Digit9, modifier::NONE)),
        ' ' => Some((KeyCode::Space, modifier::NONE)),
        '\n' => Some((KeyCode::Enter, modifier::NONE)),
        '\t' => Some((KeyCode::Tab, modifier::NONE)),
        _ => None,
    }
}
