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

        match dispatch(interrupt_fd, trimmed, &mut out) {
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

fn dispatch(fd: RawFd, input: &str, out: &mut impl Write) -> CliAction {
    // Split at most into 2 parts to preserve spaces in typed text.
    let (cmd, rest) = input
        .split_once(char::is_whitespace)
        .map(|(a, b)| (a, b.trim()))
        .unwrap_or((input, ""));

    match cmd.to_ascii_lowercase().as_str() {
        "quit" | "exit" => return CliAction::Quit,

        "help" => {
            writeln!(out, "Commands:"                                      ).ok();
            writeln!(out, "  type <text>      - type a string"             ).ok();
            writeln!(out, "  key <name>       - send named key"            ).ok();
            writeln!(out, "      (tab, enter, space, backspace, esc,"      ).ok();
            writeln!(out, "       up, down, left, right, home, end,"       ).ok();
            writeln!(out, "       pgup, pgdn, f1-f12, a-z, 0-9)"          ).ok();
            writeln!(out, "  quit / exit      - disconnect"                ).ok();
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

        "key" => {
            if rest.is_empty() {
                writeln!(out, "[cli] usage: key <name>").ok();
            } else {
                send_named_key(fd, rest, out);
            }
        }

        other => {
            writeln!(out, "[cli] unknown command: {other:?}. Type 'help'.").ok();
        }
    }

    CliAction::Continue
}

/// Resolve a key name string to a (KeyCode, modifier) pair and send it.
fn send_named_key(fd: RawFd, name: &str, out: &mut impl Write) {
    let result: Option<(KeyCode, u8)> = match name.to_ascii_lowercase().as_str() {
        "tab"       => Some((KeyCode::Tab,        modifier::NONE)),
        "enter"     => Some((KeyCode::Enter,       modifier::NONE)),
        "space"     => Some((KeyCode::Space,       modifier::NONE)),
        "backspace" => Some((KeyCode::Backspace,   modifier::NONE)),
        "esc" | "escape" => Some((KeyCode::Escape, modifier::NONE)),
        "up"        => Some((KeyCode::ArrowUp,     modifier::NONE)),
        "down"      => Some((KeyCode::ArrowDown,   modifier::NONE)),
        "left"      => Some((KeyCode::ArrowLeft,   modifier::NONE)),
        "right"     => Some((KeyCode::ArrowRight,  modifier::NONE)),
        "home"      => Some((KeyCode::Home,        modifier::NONE)),
        "end"       => Some((KeyCode::End,         modifier::NONE)),
        "pgup" | "pageup"   => Some((KeyCode::PageUp,   modifier::NONE)),
        "pgdn" | "pagedown" => Some((KeyCode::PageDown, modifier::NONE)),
        "caps" | "capslock" => Some((KeyCode::CapsLock, modifier::NONE)),
        // Letters a-z
        "a" => Some((KeyCode::A, modifier::NONE)),
        "b" => Some((KeyCode::B, modifier::NONE)),
        "c" => Some((KeyCode::C, modifier::NONE)),
        "d" => Some((KeyCode::D, modifier::NONE)),
        "e" => Some((KeyCode::E, modifier::NONE)),
        "f" => Some((KeyCode::F, modifier::NONE)),
        "g" => Some((KeyCode::G, modifier::NONE)),
        "h" => Some((KeyCode::H, modifier::NONE)),
        "i" => Some((KeyCode::I, modifier::NONE)),
        "j" => Some((KeyCode::J, modifier::NONE)),
        "k" => Some((KeyCode::K, modifier::NONE)),
        "l" => Some((KeyCode::L, modifier::NONE)),
        "m" => Some((KeyCode::M, modifier::NONE)),
        "n" => Some((KeyCode::N, modifier::NONE)),
        "o" => Some((KeyCode::O, modifier::NONE)),
        "p" => Some((KeyCode::P, modifier::NONE)),
        "q" => Some((KeyCode::Q, modifier::NONE)),
        "r" => Some((KeyCode::R, modifier::NONE)),
        "s" => Some((KeyCode::S, modifier::NONE)),
        "t" => Some((KeyCode::T, modifier::NONE)),
        "u" => Some((KeyCode::U, modifier::NONE)),
        "v" => Some((KeyCode::V, modifier::NONE)),
        "w" => Some((KeyCode::W, modifier::NONE)),
        "x" => Some((KeyCode::X, modifier::NONE)),
        "y" => Some((KeyCode::Y, modifier::NONE)),
        "z" => Some((KeyCode::Z, modifier::NONE)),
        // Digits
        "0" => Some((KeyCode::Digit0, modifier::NONE)),
        "1" => Some((KeyCode::Digit1, modifier::NONE)),
        "2" => Some((KeyCode::Digit2, modifier::NONE)),
        "3" => Some((KeyCode::Digit3, modifier::NONE)),
        "4" => Some((KeyCode::Digit4, modifier::NONE)),
        "5" => Some((KeyCode::Digit5, modifier::NONE)),
        "6" => Some((KeyCode::Digit6, modifier::NONE)),
        "7" => Some((KeyCode::Digit7, modifier::NONE)),
        "8" => Some((KeyCode::Digit8, modifier::NONE)),
        "9" => Some((KeyCode::Digit9, modifier::NONE)),
        // Function keys
        "f1"  => Some((KeyCode::F1,  modifier::NONE)),
        "f2"  => Some((KeyCode::F2,  modifier::NONE)),
        "f3"  => Some((KeyCode::F3,  modifier::NONE)),
        "f4"  => Some((KeyCode::F4,  modifier::NONE)),
        "f5"  => Some((KeyCode::F5,  modifier::NONE)),
        "f6"  => Some((KeyCode::F6,  modifier::NONE)),
        "f7"  => Some((KeyCode::F7,  modifier::NONE)),
        "f8"  => Some((KeyCode::F8,  modifier::NONE)),
        "f9"  => Some((KeyCode::F9,  modifier::NONE)),
        "f10" => Some((KeyCode::F10, modifier::NONE)),
        "f11" => Some((KeyCode::F11, modifier::NONE)),
        "f12" => Some((KeyCode::F12, modifier::NONE)),
        _ => None,
    };

    match result {
        Some((kc, mods)) => {
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
        None => {
            writeln!(out, "[cli] unknown key: {name:?}. Try 'help'.").ok();
        }
    }
}
