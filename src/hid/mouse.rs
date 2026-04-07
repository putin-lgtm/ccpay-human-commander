use std::io;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;
#[cfg(target_os = "linux")]
use libc::{c_int as RawFd, send, MSG_NOSIGNAL};

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
type RawFd = libc::c_int;

fn send_mouse_report(fd: RawFd, buttons: u8, x: i16, y: i16, wheel: i8) -> io::Result<()> {
    let report = [
        0xA1,
        0x03,
        buttons,
        (x & 0xFF) as u8,
        ((x >> 8) & 0xFF) as u8,
        (y & 0xFF) as u8,
        ((y >> 8) & 0xFF) as u8,
        wheel as u8,
    ];

    let ret = unsafe {
        send(
            fd,
            report.as_ptr() as *const libc::c_void,
            report.len(),
            MSG_NOSIGNAL,
        )
    };

    if ret < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(())
}

/// Send a swipe gesture using mouse reports.
///
/// `from_x` and `to_x` are relative X positions for the swipe, while `y`
/// is the vertical offset used for the motion path.
pub fn send_swipe(interrupt_fd: RawFd, from_x: i16, to_x: i16, y: i16) -> io::Result<()> {
    send_drag_hold(interrupt_fd, from_x, to_x, y, 80)
}

/// Send a drag gesture and hold the button at the end.
///
/// `hold_ms` defines how long the button stays held after the final move.
pub fn send_drag_hold(
    interrupt_fd: RawFd,
    from_x: i16,
    to_x: i16,
    y: i16,
    hold_ms: u64,
) -> io::Result<()> {
    let mut rng = rand::thread_rng();
    let delta_x = to_x.wrapping_sub(from_x);
    let steps = (delta_x.abs() / 32).max(4) as usize;
    let step_x = if steps > 0 { delta_x / (steps as i16) } else { delta_x };

    send_mouse_report(interrupt_fd, 0x01, 0, 0, 0)?;
    sleep(Duration::from_millis(rng.gen_range(20..40)));

    if y != 0 {
        send_mouse_report(interrupt_fd, 0x01, 0, y, 0)?;
        sleep(Duration::from_millis(rng.gen_range(20..40)));
    }

    if from_x != 0 {
        send_mouse_report(interrupt_fd, 0x01, from_x, 0, 0)?;
        sleep(Duration::from_millis(rng.gen_range(20..40)));
    }

    let mut moved_x: i16 = 0;
    for i in 0..steps {
        let remaining = delta_x.wrapping_sub(moved_x);
        let step = if i + 1 == steps { remaining } else { step_x };
        send_mouse_report(interrupt_fd, 0x01, step, 0, 0)?;
        moved_x = moved_x.wrapping_add(step);
        sleep(Duration::from_millis(rng.gen_range(10..30)));
    }

    sleep(Duration::from_millis(hold_ms.max(20)));
    send_mouse_report(interrupt_fd, 0x00, 0, 0, 0)
}

/// Send a mouse wheel movement.
///
/// Positive `amount` scrolls up; negative scrolls down.
pub fn send_wheel(interrupt_fd: RawFd, amount: i8) -> io::Result<()> {
    send_mouse_report(interrupt_fd, 0x00, 0, 0, amount)?;
    send_mouse_report(interrupt_fd, 0x00, 0, 0, 0)
}

/// Move the cursor by a relative offset (offset_x, offset_y) from the current
/// pointer position, without pressing any button.
pub fn send_move(interrupt_fd: RawFd, offset_x: i16, offset_y: i16) -> io::Result<()> {
    send_mouse_report(interrupt_fd, 0x00, offset_x, offset_y, 0)?;
    // idle report so the device registers the new focus position
    send_mouse_report(interrupt_fd, 0x00, 0, 0, 0)
}

/// Park the cursor at the top-left corner of the screen by sending a large
/// negative delta that exceeds any phone screen dimension.
/// After this call the caller should reset its virtual cursor position to (0, 0).
pub fn send_cursor_home(interrupt_fd: RawFd) -> io::Result<()> {
    // -10000 on each axis exceeds any current Android device screen dimension
    send_mouse_report(interrupt_fd, 0x00, -10000_i16, -10000_i16, 0)?;
    send_mouse_report(interrupt_fd, 0x00, 0, 0, 0)
}

/// Move the cursor by an i32 delta (handles deltas larger than i16::MAX by
/// splitting into two reports, which may arise when jumping across a large screen).
pub fn send_delta(interrupt_fd: RawFd, dx: i32, dy: i32) -> io::Result<()> {
    let cx = dx.clamp(-32767, 32767) as i16;
    let cy = dy.clamp(-32767, 32767) as i16;
    send_move(interrupt_fd, cx, cy)?;
    let rem_x = dx - cx as i32;
    let rem_y = dy - cy as i32;
    if rem_x != 0 || rem_y != 0 {
        send_move(interrupt_fd, rem_x.clamp(-32767, 32767) as i16, rem_y.clamp(-32767, 32767) as i16)?;
    }
    Ok(())
}
