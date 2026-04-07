/// Named macro sequences — mỗi macro là một chuỗi thao tác HID tự động.
///
/// Chạy macro bằng lệnh CLI: `macro <tên>`

use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

#[cfg(target_os = "linux")]
use libc::c_int as RawFd;
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
type RawFd = libc::c_int;

use crate::hid::{self, modifier, KeyCode};

/// Chạy macro theo tên trên `fd`.
pub fn run_macro(fd: RawFd, name: &str, out: &mut impl Write) -> io::Result<()> {
    match name.to_ascii_lowercase().as_str() {
        "download-image" | "dl-img" => macro_download_image(fd, out),
        _ => {
            writeln!(out, "[macro] macro không tồn tại: {name:?}. Dùng 'help' để xem danh sách.").ok();
            Ok(())
        }
    }
}

/// **download-image**: mở URL ảnh trong Chrome Android rồi tải về qua long-press.
///
/// Yêu cầu: Chrome Android đang mở và đang được focus ở màn hình chính.
///
/// Luồng thực thi:
///   1. Ctrl+L         — focus thanh địa chỉ Chrome
///   2. Ctrl+A         — chọn hết URL cũ
///   3. Gõ URL         — dán URL ảnh mới vào
///   4. Enter          — điều hướng tới ảnh (tab Chrome hiển thị full-screen)
///   5. Chờ 3.5 giây   — đợi ảnh tải xong
///   6. Long-press 1.2s — giữ chuột tại vị trí hiện tại → context menu ảnh Chrome
///   7. Chờ 600 ms      — đợi menu animation
///   8. ArrowDown       — chọn mục đầu tiên ("Save image" / "Lưu ảnh")
///   9. Enter           — xác nhận tải về
///  10. Thông báo hoàn tất — kiểm tra thư mục Downloads
fn macro_download_image(fd: RawFd, out: &mut impl Write) -> io::Result<()> {
    const URL: &str = "https://files.catbox.moe/qwt4ou.jpg";

    writeln!(out, "[macro] download-image: bắt đầu...").ok();

    // ── Bước 1: focus thanh địa chỉ ──────────────────────────────────────
    writeln!(out, "[macro] Bước 1/5: focus thanh địa chỉ Chrome (Ctrl+L)...").ok();
    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::L)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(400));

    // ── Bước 2: xóa URL cũ ───────────────────────────────────────────────
    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::A)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(150));

    // ── Bước 3: nhập URL ─────────────────────────────────────────────────
    writeln!(out, "[macro] Bước 2/5: gõ URL ảnh...").ok();
    hid::type_string(fd, URL)?;
    sleep(Duration::from_millis(200));

    // ── Bước 4: điều hướng ───────────────────────────────────────────────
    writeln!(out, "[macro] Bước 3/5: điều hướng → đợi ảnh tải (3.5s)...").ok();
    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(3500));

    // ── Bước 5: long-press để mở context menu ────────────────────────────
    writeln!(out, "[macro] Bước 4/5: long-press để mở context menu...").ok();
    // send_drag_hold(fd, 0, 0, 0, 1200) = nhấn giữ button 1 tại vị trí hiện tại
    hid::mouse::send_drag_hold(fd, 0, 0, 0, 1200)?;
    sleep(Duration::from_millis(600));

    // ── Bước 6: chọn "Lưu ảnh" (mục đầu trong context menu ảnh Chrome) ──
    writeln!(out, "[macro] Bước 5/5: chọn 'Lưu ảnh' → Enter...").ok();
    hid::key_press(fd, modifier::NONE, KeyCode::ArrowDown)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(100));
    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;

    writeln!(out, "[macro] download-image: hoàn tất! Kiểm tra thư mục Downloads.").ok();
    Ok(())
}
