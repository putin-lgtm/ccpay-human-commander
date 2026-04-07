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
        "download-image" | "dl-img"   => macro_download_image(fd, out),
        "curl-download"  | "dl-curl"  => macro_curl_download(fd, out),
        _ => {
            writeln!(out, "[macro] macro không tồn tại: {name:?}.").ok();
            writeln!(out, "[macro] Danh sách macro:").ok();
            writeln!(out, "  download-image / dl-img  — tải ảnh qua Chrome context menu").ok();
            writeln!(out, "  curl-download / dl-curl  — tải ảnh qua Termux curl (đáng tin cậy hơn)").ok();
            Ok(())
        }
    }
}

/// **download-image**: mở URL ảnh trong Chrome Android rồi tải về qua long-press
/// và sau đó mở ảnh lên ngay.
///
/// Yêu cầu: Chrome Android đang mở và đang được focus ở màn hình chính.
///
/// Luồng thực thi:
///   1. Ctrl+L          — focus thanh địa chỉ Chrome
///   2. Ctrl+A          — chọn hết URL cũ
///   3. Gõ URL          — nhập URL ảnh mới
///   4. Enter           — điều hướng tới ảnh (Chrome hiển thị full-screen)
///   5. Chờ 3.5 giây    — đợi ảnh tải xong
///   6. Long-press 1.2s — giữ chuột → context menu ảnh Chrome
///   7. Chờ 600 ms       — đợi menu animation
///   8. ArrowDown        — chọn mục đầu ("Save image" / "Lưu ảnh")
///   9. Enter            — xác nhận tải về
///  10. Chờ 2.5 giây    — đợi download hoàn tất
///  11. Ctrl+L → gõ chrome://downloads → Enter — mở trang Downloads Chrome
///  12. Chờ 1 giây
///  13. Tab → Enter     — focus và mở file đầu tiên trong danh sách
fn macro_download_image(fd: RawFd, out: &mut impl Write) -> io::Result<()> {
    const URL: &str = "https://files.catbox.moe/qwt4ou.jpg";
    const DOWNLOADS_PAGE: &str = "chrome://downloads";

    writeln!(out, "[macro] download-image: bắt đầu...").ok();

    // ── Bước 1: focus thanh địa chỉ ──────────────────────────────────────
    writeln!(out, "[macro] Bước 1/6: focus thanh địa chỉ Chrome (Ctrl+L)...").ok();
    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::L)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(400));

    // ── Bước 2: xóa URL cũ ───────────────────────────────────────────────
    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::A)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(150));

    // ── Bước 3: nhập URL ─────────────────────────────────────────────────
    writeln!(out, "[macro] Bước 2/6: gõ URL ảnh...").ok();
    hid::type_string(fd, URL)?;
    sleep(Duration::from_millis(200));

    // ── Bước 4: điều hướng ───────────────────────────────────────────────
    writeln!(out, "[macro] Bước 3/6: điều hướng → đợi ảnh tải (3.5s)...").ok();
    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(3500));

    // ── Bước 5: long-press để mở context menu ────────────────────────────
    writeln!(out, "[macro] Bước 4/6: long-press để mở context menu...").ok();
    hid::mouse::send_drag_hold(fd, 0, 0, 0, 1200)?;
    sleep(Duration::from_millis(600));

    // ── Bước 6: chọn "Lưu ảnh" ───────────────────────────────────────────
    writeln!(out, "[macro] Bước 5/6: chọn 'Lưu ảnh' → Enter...").ok();
    hid::key_press(fd, modifier::NONE, KeyCode::ArrowDown)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(100));
    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;

    // ── Đợi download hoàn tất ────────────────────────────────────────────
    writeln!(out, "[macro] Bước 6/6: đợi download (2.5s) rồi mở ảnh...").ok();
    sleep(Duration::from_millis(2500));

    // ── Mở chrome://downloads ────────────────────────────────────────────
    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::L)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(400));

    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::A)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(150));

    hid::type_string(fd, DOWNLOADS_PAGE)?;
    sleep(Duration::from_millis(150));

    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(1000));

    // ── Focus file đầu tiên và mở ─────────────────────────────────────────
    // Tab đưa focus đến item đầu tiên trong danh sách Downloads Chrome
    hid::key_press(fd, modifier::NONE, KeyCode::Tab)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(200));
    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;

    writeln!(out, "[macro] download-image: hoàn tất! Ảnh đã được mở.").ok();
    Ok(())
}

/// **curl-download**: tự mở Termux, tải ảnh bằng `curl` rồi mở bằng `termux-open`.
///
/// Đây là giải pháp đáng tin cậy hơn macro Chrome UI vì không phụ thuộc vào
/// vị trí context menu hay animation timing.
///
/// **Yêu cầu**: `termux-setup-storage` đã được chạy ít nhất một lần.
///
/// Luồng thực thi:
///   1. Consumer Home     — về màn hình chính
///   2. Gõ "termux"       — launcher Android tìm kiếm app (hầu hết launcher hỗ trợ)
///   3. Enter             — mở Termux
///   4. Chờ 2s            — đợi Termux khởi động
///   5. Ctrl+C            — hủy lệnh đang chạy nếu có
///   6. Gõ lệnh curl && termux-open
///   7. Enter             — thực thi
fn macro_curl_download(fd: RawFd, out: &mut impl Write) -> io::Result<()> {
    const URL: &str = "https://files.catbox.moe/qwt4ou.jpg";
    const DEST: &str = "~/storage/downloads/qwt4ou.jpg";

    writeln!(out, "[macro] curl-download: bắt đầu...").ok();

    // ── Bước 1: về màn hình chính ─────────────────────────────────────────
    writeln!(out, "[macro] Bước 1/4: về màn hình chính...").ok();
    hid::consumer_key_tap(fd, hid::ConsumerKey::Home)?;
    sleep(Duration::from_millis(600));

    // ── Bước 2+3: tìm kiếm và mở Termux ──────────────────────────────────
    // Hầu hết launcher (Pixel, One UI, Nova...) khi gõ trên home screen
    // sẽ tự động mở search và lọc app theo tên.
    writeln!(out, "[macro] Bước 2/4: tìm kiếm app 'termux'...").ok();
    hid::type_string(fd, "termux")?;
    sleep(Duration::from_millis(1200));

    writeln!(out, "[macro] Bước 3/4: mở Termux...").ok();
    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(2000)); // đợi Termux load

    // ── Bước 4: chạy lệnh curl ────────────────────────────────────────────
    // Ctrl+C trước để hủy lệnh cũ nếu có
    hid::key_press(fd, modifier::LEFT_CTRL, KeyCode::C)?;
    hid::key_release(fd)?;
    sleep(Duration::from_millis(150));

    writeln!(out, "[macro] Bước 4/4: gõ lệnh curl...").ok();
    let cmd = format!("curl -L -o {DEST} '{URL}' && termux-open {DEST}");
    hid::type_string(fd, &cmd)?;
    sleep(Duration::from_millis(200));

    hid::key_press(fd, modifier::NONE, KeyCode::Enter)?;
    hid::key_release(fd)?;

    writeln!(out, "[macro] curl-download: lệnh đã gửi. Chờ tải xong...").ok();
    writeln!(out, "[macro] Ảnh sẽ tự mở qua Gallery sau khi curl hoàn tất.").ok();
    Ok(())
}
