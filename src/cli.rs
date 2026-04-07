/// Interactive CLI command loop.
///
/// Reads lines from stdin and translates them into HID keystrokes sent
/// over the L2CAP interrupt socket.  Designed to be minimal and allocation-
/// efficient — it reuses a single `String` buffer for all input.
///
/// ## Available commands
///
/// | Input            | Action                                      |
/// |------------------|---------------------------------------------|
/// | `type <text>`    | Type the literal string character-by-character |
/// | `key tab`        | Send a single Tab keystroke                 |
/// | `key enter`      | Send Enter                                  |
/// | `key space`      | Send Space                                  |
/// | `key backspace`  | Send Backspace                              |
/// | `key esc`        | Send Escape                                 |
/// | `key up/down/left/right` | Arrow keys                         |
/// | `key home/end/pgup/pgdn` | Navigation keys                    |
/// | `key f1`…`key f12` | Function keys                            |
/// | `quit` / `exit`  | Disconnect and exit                         |

use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};
#[cfg(target_os = "linux")]
use libc::c_int as RawFd;
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
type RawFd = libc::c_int;

use crate::hid::{self, KeyCode, modifier, type_string};

/// Run the interactive command loop until the user types `quit` or `exit`,
/// or stdin is closed.
///
/// `interrupt_fd` is the raw L2CAP fd for the HID Interrupt channel.
pub fn run_cli(interrupt_fd: RawFd) {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut line = String::with_capacity(256);
    let mut last_screenshot = Instant::now() - Duration::from_secs(1);

    writeln!(out, "ccpay-human-commander ready. Type 'help' for commands.").ok();

    loop {
        write!(out, "> ").ok();
        out.flush().ok();

        line.clear();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Err(e) => {
                eprintln!("[cli] read error: {e}");
                break;
            }
            Ok(_) => {}
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match dispatch(interrupt_fd, trimmed, &mut out, &mut last_screenshot) {
            CliAction::Continue => {}
            CliAction::Quit => break,
        }
    }

    writeln!(out, "Disconnecting.").ok();
}

enum CliAction {
    Continue,
    Quit,
}

fn dispatch(fd: RawFd, input: &str, out: &mut impl Write, last_screenshot: &mut Instant) -> CliAction {
    // Split at most into 2 parts to preserve spaces in typed text.
    let (cmd, rest) = input
        .split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim()))
        .unwrap_or((input, ""));

    match cmd.to_ascii_lowercase().as_str() {
        "quit" | "exit" => return CliAction::Quit,

        "help" => {
            writeln!(out, "Commands:"                                                  ).ok();
            writeln!(out, "  type <text>            - gõ chuỗi ký tự"                      ).ok();
            writeln!(out, "  key <name>             - gửi phím theo tên"               ).ok();
            writeln!(out, ""                                                           ).ok();
            writeln!(out, "  Phím Android (Consumer Control):"                        ).ok();
            writeln!(out, "    back                 - phím Back (KEYCODE_BACK)"        ).ok();
            writeln!(out, "    recent               - phím Recent Apps (KEYCODE_APP_SWITCH)").ok();
            writeln!(out, "    home-android         - về màn hình chính (KEYCODE_HOME)" ).ok();
            writeln!(out, "    volup / voldown       - âm lượng"                         ).ok();
            writeln!(out, ""                                                           ).ok();
            writeln!(out, "  Swipe chuột ảo:"                                         ).ok();
            writeln!(out, "    swipe <from_x> <to_x> <y> - vuốt ảo bằng HID Mouse"     ).ok();
            writeln!(out, "    drag <from_x> <to_x> <y> <hold_ms> - vuốt giữ và thả"   ).ok();
            writeln!(out, "    wheel <delta> - cuộn bánh xe chuột"                    ).ok();
            writeln!(out, "    move <offset_x> <offset_y> - di chuyển con trỏ tương đối").ok();
            writeln!(out, ""                                                           ).ok();
            writeln!(out, "  Chụp màn hình Samsung:"                                    ).ok();
            writeln!(out, "    ss / screenshot      - Phím Print Screen (Samsung One UI)").ok();
            writeln!(out, "    ss-samsung           - Ctrl+Shift+S (Samsung + bàn phím BT)").ok();
            writeln!(out, ""                                                           ).ok();
            writeln!(out, "  Phím bàn phím thông thường:"                              ).ok();
            writeln!(out, "    tab, enter, space, backspace, esc"                     ).ok();
            writeln!(out, "    up/down/left/right, home/end, pgup/pgdn"               ).ok();
            writeln!(out, "    f1-f12, a-z, 0-9"                                      ).ok();
            writeln!(out, "  quit / exit            - ngắt kết nối"                    ).ok();
        }

        "type" => {
            if rest.is_empty() {
                writeln!(out, "[cli] usage: type <text>").ok();
            } else {
                if let Err(e) = type_string(fd, rest) {
                    eprintln!("[cli] send error: {e}");
                } else {
                    writeln!(out, "[cli] typed: {rest:?}").ok();
                }
            }
        }

        "swipe" => {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() != 3 {
                writeln!(out, "[cli] usage: swipe <from_x> <to_x> <y>").ok();
            } else if let (Ok(from_x), Ok(to_x), Ok(y)) = (parts[0].parse::<i16>(), parts[1].parse::<i16>(), parts[2].parse::<i16>()) {
                if let Err(e) = hid::mouse::send_swipe(fd, from_x, to_x, y) {
                    eprintln!("[cli] swipe error: {e}");
                } else {
                    writeln!(out, "[cli] swipe sent: {from_x} -> {to_x} @ {y}").ok();
                }
            } else {
                writeln!(out, "[cli] swipe parameters must be integers").ok();
            }
        }

        "drag" => {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() != 4 {
                writeln!(out, "[cli] usage: drag <from_x> <to_x> <y> <hold_ms>").ok();
            } else if let (Ok(from_x), Ok(to_x), Ok(y), Ok(hold_ms)) = (
                parts[0].parse::<i16>(),
                parts[1].parse::<i16>(),
                parts[2].parse::<i16>(),
                parts[3].parse::<u64>(),
            ) {
                if let Err(e) = hid::mouse::send_drag_hold(fd, from_x, to_x, y, hold_ms) {
                    eprintln!("[cli] drag error: {e}");
                } else {
                    writeln!(out, "[cli] drag sent: {from_x} -> {to_x} @ {y}, hold {hold_ms}ms").ok();
                }
            } else {
                writeln!(out, "[cli] drag parameters must be integers").ok();
            }
        }

        "wheel" => {
            if rest.is_empty() {
                writeln!(out, "[cli] usage: wheel <delta>").ok();
            } else if let Ok(delta) = rest.parse::<i8>() {
                if let Err(e) = hid::mouse::send_wheel(fd, delta) {
                    eprintln!("[cli] wheel error: {e}");
                } else {
                    writeln!(out, "[cli] wheel sent: {delta}").ok();
                }
            } else {
                writeln!(out, "[cli] wheel delta must be a signed integer").ok();
            }
        }

        "move" => {
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() != 2 {
                writeln!(out, "[cli] usage: move <offset_x> <offset_y>").ok();
            } else if let (Ok(dx), Ok(dy)) = (parts[0].parse::<i16>(), parts[1].parse::<i16>()) {
                if let Err(e) = hid::mouse::send_move(fd, dx, dy) {
                    eprintln!("[cli] move error: {e}");
                } else {
                    writeln!(out, "[cli] move sent: dx={dx} dy={dy}").ok();
                }
            } else {
                writeln!(out, "[cli] move offsets must be signed integers").ok();
            }
        }

        "key" => {
            if rest.is_empty() {
                writeln!(out, "[cli] usage: key <name>").ok();
            } else {
                send_named_key(fd, rest, out, last_screenshot);
            }
        }

        other => {
            writeln!(out, "[cli] unknown command: {other:?}. Type 'help'.").ok();
        }
    }

    CliAction::Continue
}

/// Internal key action — distinguishes keyboard keys, consumer control keys, and macros.
enum KeyAction {
    Keyboard(KeyCode, u8),
    Consumer(hid::ConsumerKey),
    SamsungScreenshot,
}

/// Resolve a key name string and dispatch the appropriate HID report(s).
fn send_named_key(fd: RawFd, name: &str, out: &mut impl Write, last_screenshot: &mut Instant) {
    let action: Option<KeyAction> = match name.to_ascii_lowercase().as_str() {
        // ── Phím Android (HID Consumer Control, Report ID 2) ───────────────────
        "back"                                  => Some(KeyAction::Consumer(hid::ConsumerKey::Back)),
        "recent" | "recents" | "overview"       => Some(KeyAction::Consumer(hid::ConsumerKey::Recent)),
        "home-android" | "androidhome"          => Some(KeyAction::Consumer(hid::ConsumerKey::Home)),
        "volup"   | "volumeup"                  => Some(KeyAction::Consumer(hid::ConsumerKey::VolumeUp)),
        "voldown" | "volumedown"                => Some(KeyAction::Consumer(hid::ConsumerKey::VolumeDown)),

        // ── Chụp màn hình Samsung ─────────────────────────────────────────────
        // Print Screen → KEYCODE_SYSRQ trên Android, Samsung One UI ánh xạ sang screenshot
        "ss" | "screenshot" | "prtsc" | "printscreen"
                                                => Some(KeyAction::Keyboard(KeyCode::PrintScreen, modifier::NONE)),
        // Ctrl+Shift+S macro — Samsung One UI khi kết nối bàn phím Bluetooth
        "ss-samsung" | "screenshot-samsung"     => Some(KeyAction::SamsungScreenshot),

        // ── Phím bàn phím thông thường (HID Keyboard page, Report ID 1) ────────
        "tab"       => Some(KeyAction::Keyboard(KeyCode::Tab,        modifier::NONE)),
        "enter"     => Some(KeyAction::Keyboard(KeyCode::Enter,       modifier::NONE)),
        "space"     => Some(KeyAction::Keyboard(KeyCode::Space,       modifier::NONE)),
        "backspace" => Some(KeyAction::Keyboard(KeyCode::Backspace,   modifier::NONE)),
        "esc" | "escape" => Some(KeyAction::Keyboard(KeyCode::Escape, modifier::NONE)),
        "up"        => Some(KeyAction::Keyboard(KeyCode::ArrowUp,     modifier::NONE)),
        "down"      => Some(KeyAction::Keyboard(KeyCode::ArrowDown,   modifier::NONE)),
        "left"      => Some(KeyAction::Keyboard(KeyCode::ArrowLeft,   modifier::NONE)),
        "right"     => Some(KeyAction::Keyboard(KeyCode::ArrowRight,  modifier::NONE)),
        "home"      => Some(KeyAction::Keyboard(KeyCode::Home,        modifier::NONE)),
        "end"       => Some(KeyAction::Keyboard(KeyCode::End,         modifier::NONE)),
        "pgup" | "pageup"   => Some(KeyAction::Keyboard(KeyCode::PageUp,   modifier::NONE)),
        "pgdn" | "pagedown" => Some(KeyAction::Keyboard(KeyCode::PageDown, modifier::NONE)),
        "caps" | "capslock" => Some(KeyAction::Keyboard(KeyCode::CapsLock, modifier::NONE)),
        // Letters a-z
        "a" => Some(KeyAction::Keyboard(KeyCode::A, modifier::NONE)),
        "b" => Some(KeyAction::Keyboard(KeyCode::B, modifier::NONE)),
        "c" => Some(KeyAction::Keyboard(KeyCode::C, modifier::NONE)),
        "d" => Some(KeyAction::Keyboard(KeyCode::D, modifier::NONE)),
        "e" => Some(KeyAction::Keyboard(KeyCode::E, modifier::NONE)),
        "f" => Some(KeyAction::Keyboard(KeyCode::F, modifier::NONE)),
        "g" => Some(KeyAction::Keyboard(KeyCode::G, modifier::NONE)),
        "h" => Some(KeyAction::Keyboard(KeyCode::H, modifier::NONE)),
        "i" => Some(KeyAction::Keyboard(KeyCode::I, modifier::NONE)),
        "j" => Some(KeyAction::Keyboard(KeyCode::J, modifier::NONE)),
        "k" => Some(KeyAction::Keyboard(KeyCode::K, modifier::NONE)),
        "l" => Some(KeyAction::Keyboard(KeyCode::L, modifier::NONE)),
        "m" => Some(KeyAction::Keyboard(KeyCode::M, modifier::NONE)),
        "n" => Some(KeyAction::Keyboard(KeyCode::N, modifier::NONE)),
        "o" => Some(KeyAction::Keyboard(KeyCode::O, modifier::NONE)),
        "p" => Some(KeyAction::Keyboard(KeyCode::P, modifier::NONE)),
        "q" => Some(KeyAction::Keyboard(KeyCode::Q, modifier::NONE)),
        "r" => Some(KeyAction::Keyboard(KeyCode::R, modifier::NONE)),
        "s" => Some(KeyAction::Keyboard(KeyCode::S, modifier::NONE)),
        "t" => Some(KeyAction::Keyboard(KeyCode::T, modifier::NONE)),
        "u" => Some(KeyAction::Keyboard(KeyCode::U, modifier::NONE)),
        "v" => Some(KeyAction::Keyboard(KeyCode::V, modifier::NONE)),
        "w" => Some(KeyAction::Keyboard(KeyCode::W, modifier::NONE)),
        "x" => Some(KeyAction::Keyboard(KeyCode::X, modifier::NONE)),
        "y" => Some(KeyAction::Keyboard(KeyCode::Y, modifier::NONE)),
        "z" => Some(KeyAction::Keyboard(KeyCode::Z, modifier::NONE)),
        // Digits
        "0" => Some(KeyAction::Keyboard(KeyCode::Digit0, modifier::NONE)),
        "1" => Some(KeyAction::Keyboard(KeyCode::Digit1, modifier::NONE)),
        "2" => Some(KeyAction::Keyboard(KeyCode::Digit2, modifier::NONE)),
        "3" => Some(KeyAction::Keyboard(KeyCode::Digit3, modifier::NONE)),
        "4" => Some(KeyAction::Keyboard(KeyCode::Digit4, modifier::NONE)),
        "5" => Some(KeyAction::Keyboard(KeyCode::Digit5, modifier::NONE)),
        "6" => Some(KeyAction::Keyboard(KeyCode::Digit6, modifier::NONE)),
        "7" => Some(KeyAction::Keyboard(KeyCode::Digit7, modifier::NONE)),
        "8" => Some(KeyAction::Keyboard(KeyCode::Digit8, modifier::NONE)),
        "9" => Some(KeyAction::Keyboard(KeyCode::Digit9, modifier::NONE)),
        // Function keys
        "f1"  => Some(KeyAction::Keyboard(KeyCode::F1,  modifier::NONE)),
        "f2"  => Some(KeyAction::Keyboard(KeyCode::F2,  modifier::NONE)),
        "f3"  => Some(KeyAction::Keyboard(KeyCode::F3,  modifier::NONE)),
        "f4"  => Some(KeyAction::Keyboard(KeyCode::F4,  modifier::NONE)),
        "f5"  => Some(KeyAction::Keyboard(KeyCode::F5,  modifier::NONE)),
        "f6"  => Some(KeyAction::Keyboard(KeyCode::F6,  modifier::NONE)),
        "f7"  => Some(KeyAction::Keyboard(KeyCode::F7,  modifier::NONE)),
        "f8"  => Some(KeyAction::Keyboard(KeyCode::F8,  modifier::NONE)),
        "f9"  => Some(KeyAction::Keyboard(KeyCode::F9,  modifier::NONE)),
        "f10" => Some(KeyAction::Keyboard(KeyCode::F10, modifier::NONE)),
        "f11" => Some(KeyAction::Keyboard(KeyCode::F11, modifier::NONE)),
        "f12" => Some(KeyAction::Keyboard(KeyCode::F12, modifier::NONE)),
        _ => None,
    };

    match action {
        Some(KeyAction::Keyboard(kc, mods)) => {
            if let Err(e) = hid::key_press(fd, mods, kc) {
                eprintln!("[cli] press error: {e}");
                return;
            }
            if let Err(e) = hid::key_release(fd) {
                eprintln!("[cli] release error: {e}");
                return;
            }
            writeln!(out, "[cli] key sent: {name}").ok();
        }
        Some(KeyAction::Consumer(ck)) => {
            if let Err(e) = hid::consumer_key_tap(fd, ck) {
                eprintln!("[cli] consumer key error: {e}");
                return;
            }
            writeln!(out, "[cli] key sent: {name}").ok();
        }
        Some(KeyAction::SamsungScreenshot) => {
            let now = Instant::now();
            if now.duration_since(*last_screenshot) < Duration::from_secs(1) {
                writeln!(out, "[cli] screenshot rate limit: wait at least 1s between screenshot commands").ok();
                return;
            }
            *last_screenshot = now;
            if let Err(e) = hid::samsung_screenshot(fd) {
                eprintln!("[cli] screenshot error: {e}");
                return;
            }
            writeln!(out, "[cli] Samsung screenshot sent (Ctrl+Shift+S)").ok();
        }
        None => {
            writeln!(out, "[cli] unknown key: {name:?}. Type 'help'.").ok();
        }
    }
}
