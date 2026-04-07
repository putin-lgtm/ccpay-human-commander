# ccpay-human-commander

> Ứng dụng Linux viết bằng Rust — giả lập bàn phím Bluetooth HID để điều khiển thiết bị Android từ xa.

---

## Dự án này làm gì?

`ccpay-human-commander` biến máy tính Linux của bạn thành một **bàn phím Bluetooth không dây ảo**. Bạn gõ lệnh trên terminal, ứng dụng sẽ gửi các tổ hợp phím thật sự đến điện thoại Android (hoặc bất kỳ thiết bị Bluetooth nào hỗ trợ HID) thông qua kết nối Bluetooth chuẩn.

### Ứng dụng thực tế
- Điều hướng điện thoại Android từ máy tính (Tab, Enter, mũi tên, v.v.)
- Nhập văn bản tự động vào thiết bị từ xa
- Kiểm thử giao diện người dùng trên thiết bị di động mà không cần chạm tay
- Hỗ trợ tiếp cận (accessibility) cho người dùng khuyết tật

---

## Kiến trúc tổng quan

```
┌─────────────────────────────────────────────────────────┐
│                  ccpay-human-commander                  │
│                                                         │
│  ┌──────────┐   ┌──────────┐   ┌────────┐   ┌───────┐  │
│  │  cli.rs  │──▶│  hid.rs  │──▶│l2cap.rs│──▶│ BlueZ │  │
│  │ (CLI)    │   │ (Report) │   │(Socket)│   │ Stack │  │
│  └──────────┘   └──────────┘   └────────┘   └───┬───┘  │
│                                                  │      │
│  ┌────────────┐   ┌──────────┐            D-Bus  │      │
│  │ profile.rs │──▶│  sdp.rs  │◀──────────────────┘      │
│  │ (D-Bus)    │   │ (SDP XML)│                          │
│  └────────────┘   └──────────┘                          │
└─────────────────────────────────────────────────────────┘
                          │
                    Bluetooth BR/EDR
                          │
               ┌──────────────────────┐
               │  Android F4:7D:EF:.. │
               │  (thiết bị đích)     │
               └──────────────────────┘
```

---

## Cấu trúc mã nguồn

| File | Vai trò |
|------|---------|
| `src/main.rs` | Điểm khởi đầu — kết nối tất cả các bước lại với nhau |
| `src/sdp.rs` | XML mô tả dịch vụ SDP — giúp Android nhận diện thiết bị là "Standard Bluetooth Keyboard" |
| `src/profile.rs` | Đăng ký HID profile với BlueZ thông qua D-Bus (`org.bluez.ProfileManager1`) |
| `src/l2cap.rs` | Tạo và kết nối raw L2CAP socket qua `libc` (PSM 17 — Control, PSM 19 — Interrupt) |
| `src/hid.rs` | Định dạng và gửi HID Input Report 8 byte; ánh xạ ký tự ASCII → keycode USB HID |
| `src/hid/mouse.rs` | Định dạng và gửi HID Mouse Report; hỗ trợ vuốt ảo (swipe) |
| `src/cli.rs` | Vòng lặp lệnh tương tác — nhận lệnh từ stdin và gửi phím tương ứng |

---

## Luồng hoạt động

```
1. Khởi động
   └─▶ Kết nối System D-Bus
   └─▶ Gọi BlueZ RegisterProfile (UUID 0x1124 — HID)
       └─▶ BlueZ đăng ký SDP record → Android thấy "Standard Bluetooth Keyboard"

2. Kết nối L2CAP
   └─▶ Mở socket AF_BLUETOOTH / SOCK_SEQPACKET
   └─▶ connect() tới PSM 17 (HID Control)
   └─▶ connect() tới PSM 19 (HID Interrupt)

3. Vòng lặp CLI
   └─▶ Đọc lệnh từ stdin
   └─▶ Chuyển đổi → HID Input Report [0xA1, modifier, 0x00, keycode, 0, 0, 0, 0]
   └─▶ Gửi qua interrupt socket bằng libc::send()
   └─▶ Gửi key-release report [0xA1, 0, 0, 0, 0, 0, 0, 0]

4. Thoát
   └─▶ Hủy đăng ký profile với BlueZ
   └─▶ Đóng cả hai L2CAP socket
```

---

## Yêu cầu hệ thống

- **OS**: Linux (kernel ≥ 4.x)
- **Bluetooth**: BlueZ ≥ 5.50 (`bluetoothd` đang chạy)
- **Rust**: ≥ 1.75 (edition 2021)
- **Quyền**: `CAP_NET_RAW` hoặc chạy với `sudo` để tạo raw Bluetooth socket
- **Typing fuzz**: gom random delay mỗi ký tự để tránh mẫu gõ quá đều và phát hiện thao tác tự động.
- **Ẩn dấu vết**: Trên Ubuntu, tắt log `bluetoothd` hoặc cấu hình `LogLevel = none` trong `/etc/bluetooth/main.conf` để tránh ghi lại MAC và thời gian thao tác.

### Cài đặt BlueZ (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install bluez bluetooth
sudo systemctl enable --now bluetooth
```

---

## Cách build và chạy dự án

### Cách 1 — Dùng `cargo run` (khợn nhất, dùng khi phát triển)

```bash
cd ccpay-human-commander

# Chạy bản debug (tự động build rồi chạy)
sudo $(which cargo) run

# Hoặc bản release (chậm hơn lần đầu do tối ưu, nhưng nhanh hơn khi chạy)
sudo $(which cargo) run --release
```

> **Tại sao dùng `$(which cargo)` thay vì chỉ `cargo`?**  
> Khi gõ `sudo cargo run`, `sudo` chạy với `PATH` rút gọn — có thể không tìm thấy `cargo`. Dùng `$(which cargo)` để truyền đường dẫn tuyệt đối.

### Cách 2 — Build trước, chạy sau (dùng khi deploy)

```bash
# Bước 1: Build
cargo build --release

# Bước 2: Chạy binary (đã tồn tại sau khi build)
sudo ./target/release/ccpay-human-commander
```

> **Lưu ý**: Phải chạy `cargo build --release` ít nhất một lần trước. Nếu chạy `sudo ./target/release/...` mà chưa build → báo `command not found`.

### Cách 3 — Không cần sudo (cấp capability)

```bash
cargo build --release

# Cấp quyền một lần
sudo setcap cap_net_raw+ep ./target/release/ccpay-human-commander

# Sau đó chạy bình thường không cần sudo
./target/release/ccpay-human-commander
```

```bash
# Mở bluetoothctl
sudo bluetoothctl

# Bật scan
scan on

# Sau khi thấy thiết bị, ghép đôi (thay MAC bằng địa chỉ thật)
pair F4:7D:EF:8A:3B:5C
trust F4:7D:EF:8A:3B:5C
connect F4:7D:EF:8A:3B:5C

# Thoát
exit
```

### Bước 2 — Chạy ứng dụng

```bash
# Cách khợn nhất (tự build + chạy)
sudo $(which cargo) run --release
```

Hoặc nếu đã build rồi:

```bash
sudo ./target/release/ccpay-human-commander
```

### Bước 3 — Sử dụng CLI

Sau khi kết nối thành công, dấu nhắc `>` xuất hiện:

```
ccpay-human-commander — Bluetooth HID Keyboard Emulator
Target device: F4:7D:EF:8A:3B:5C
[dbus] HID profile registered: 00001124-0000-1000-8000-00805f9b34fb
[main] Connecting L2CAP channels to F4:7D:EF:8A:3B:5C …
[main] HID channels connected (ctrl_fd=4, intr_fd=5)
ccpay-human-commander ready. Type 'help' for commands.
> 
```

---

## Danh sách lệnh CLI

### Phím Android chuyên dụng (HID Consumer Control)

> Các phím này dùng **Consumer Control Report (Report ID 2)** — khác với keyboard thông thường.
> Android nhận diện chúng như phím cứng trên điện thoại.

| Lệnh | Mô tả | Android Keycode | HID Usage |
|------|-------|-----------------|-----------|
| `key back` | Phím Back — quay lại màn hình trước | `KEYCODE_BACK` | `0x0224` AC Back |
| `key recent` | Phím Recent Apps — mở danh sách ứng dụng đang chạy | `KEYCODE_APP_SWITCH` | `0x029F` AC Show All Windows |
| `key home-android` | Về màn hình chính | `KEYCODE_HOME` | `0x0223` AC Home |
| `key volup` | Tăng âm lượng | `KEYCODE_VOLUME_UP` | `0x00E9` |
| `key voldown` | Giảm âm lượng | `KEYCODE_VOLUME_DOWN` | `0x00EA` |

### Chụp màn hình Samsung

| Lệnh | Cơ chế | Ghi chú |
|------|--------|---------|
| `key ss` hoặc `key screenshot` | Print Screen (`0x46`) → `KEYCODE_SYSRQ` | Samsung One UI tự ánh xạ sang screenshot |
| `key ss-samsung` | Macro Ctrl+Shift+S | Samsung One UI khi kết nối bàn phím BT, ổn định hơn |

> **Lưu ý**: Thử `key ss` trước. Nếu không hoạt động (tùy phiên bản One UI), dùng `key ss-samsung`.

### Phím bàn phím thông thường (HID Keyboard Report ID 1)

| Lệnh | Mô tả | Ví dụ |
|------|-------|-------|
| `type <văn bản>` | Gõ chuỗi ký tự | `type Hello World` |
| `key tab` | Phím Tab | `key tab` |
| `key enter` | Phím Enter | `key enter` |
| `key space` | Phím Space | `key space` |
| `key backspace` | Phím Backspace | `key backspace` |
| `key esc` | Phím Escape | `key esc` |
| `key up/down/left/right` | Phím mũi tên | `key up` |
| `key home / end` | Phím Home / End | `key home` |
| `key pgup / pgdn` | Page Up / Page Down | `key pgdn` |
| `key f1` … `key f12` | Phím chức năng | `key f5` |
| `key a` … `key z` | Phím chữ cái | `key a` |
| `key 0` … `key 9` | Phím số | `key 3` |
| `swipe <from_x> <to_x> <y>` | Vuốt ảo bằng HID Mouse | `swipe 0 400 20` |
| `drag <from_x> <to_x> <y> <hold_ms>` | Vuốt giữ và thả | `drag 0 400 20 120` |
| `wheel <delta>` | Cuộn bánh xe chuột | `wheel -3` |
| `help` | Hiển thị trợ giúp | `help` |
| `quit` hoặc `exit` | Ngắt kết nối và thoát | `quit` |

> **Lưu ý**: `ss` / `ss-samsung` bị giới hạn tần suất 1 lần mỗi giây để tránh trigger hệ thống phòng thủ.

### Ví dụ phiên sử dụng thực tế

```
> type ccpay
[cli] typed: "ccpay"

> key enter
[cli] key sent: enter

> key tab
[cli] key sent: tab

> key back
[cli] key sent: back

> key recent
[cli] key sent: recent

> key ss
[cli] key sent: ss

> key ss-samsung
[cli] Samsung screenshot sent (Ctrl+Shift+S)

> `drag 0 400 20 120`
[cli] drag sent: 0 -> 400 @ 20, hold 120ms

> `wheel -3`
[cli] wheel sent: -3

> quit
Disconnecting.
[dbus] HID profile unregistered
[main] Shutdown complete.
```

---

## Cách kiểm thử (Testing)

Dự án này tương tác trực tiếp với phần cứng Bluetooth nên không thể unit test toàn bộ pipeline. Tuy nhiên có thể kiểm tra từng phần:

### 1. Kiểm tra build không lỗi

```bash
cargo check
cargo build
```

### 2. Kiểm tra parse địa chỉ MAC

Hàm `parse_bdaddr` trong `l2cap.rs` có thể test thủ công bằng cách thêm vào `main.rs` tạm thời hoặc dùng `cargo test`:

```bash
# Chạy test (nếu có)
cargo test
```

Để thêm unit test vào `l2cap.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::parse_bdaddr;

    #[test]
    fn test_parse_bdaddr_valid() {
        let result = parse_bdaddr("F4:7D:EF:8A:3B:5C").unwrap();
        // Byte order bị đảo ngược (little-endian)
        assert_eq!(result, [0x5C, 0x3B, 0x8A, 0xEF, 0x7D, 0xF4]);
    }

    #[test]
    fn test_parse_bdaddr_invalid() {
        assert!(parse_bdaddr("ZZ:00:00:00:00:00").is_err());
        assert!(parse_bdaddr("00:11:22:33").is_err());
    }
}
```

### 3. Kiểm tra HID Report format

```rust
#[cfg(test)]
mod tests {
    use super::{build_report, build_release_report, modifier};

    #[test]
    fn test_key_press_report() {
        let report = build_report(modifier::LEFT_SHIFT, 0x04); // Shift+A
        assert_eq!(report[0], 0xA1); // HID input report header
        assert_eq!(report[1], 0x02); // LEFT_SHIFT modifier
        assert_eq!(report[3], 0x04); // keycode A
    }

    #[test]
    fn test_key_release_report() {
        let report = build_release_report();
        assert_eq!(report, [0xA1, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    }
}
```

### 4. Kiểm tra kết nối thực tế với Android

Trên điện thoại Android:
1. Vào **Cài đặt → Bluetooth** → đảm bảo máy tính đã được ghép đôi
2. Chạy ứng dụng
3. Mở bất kỳ trường nhập liệu nào trên Android (ví dụ: Google Search)
4. Gõ lệnh `type hello` trong CLI
5. Kiểm tra chữ "hello" xuất hiện trên màn hình Android

### 5. Kiểm tra D-Bus profile (không cần thiết bị Bluetooth)

```bash
# Kiểm tra bluetoothd đang chạy
systemctl status bluetooth

# Xem profile đã đăng ký chưa (sau khi chạy app)
sudo dbus-send --system --print-reply \
  --dest=org.bluez /org/bluez \
  org.freedesktop.DBus.Introspectable.Introspect
```

### 6. Debug với btmon (Bluetooth monitor)

```bash
# Terminal 1: theo dõi gói tin Bluetooth raw
sudo btmon

# Terminal 2: chạy ứng dụng
sudo ./target/release/ccpay-human-commander
```

---

## Công nghệ sử dụng

| Thư viện | Phiên bản | Mục đích |
|----------|-----------|----------|
| `zbus` | 4.x | Giao tiếp D-Bus với BlueZ |
| `tokio` | 1.x | Async runtime cho D-Bus |
| `libc` | 0.2.x | Raw syscall: socket, connect, send |
| `zvariant` | 4.x | Serialize dữ liệu D-Bus (Value, OwnedObjectPath) |

---

## Giải thích kỹ thuật nhanh

### HID Input Report là gì?
Đây là gói dữ liệu 8 byte theo chuẩn USB HID (được tái sử dụng trong Bluetooth HID):

```
Byte 0: 0xA1        → loại report (Input Report)
Byte 1: modifier    → bitmask phím đặc biệt (Shift, Ctrl, Alt, ...)
Byte 2: 0x00        → reserved
Byte 3: keycode     → mã phím USB HID (ví dụ: 0x28 = Enter, 0x2B = Tab)
Byte 4-7: 0x00      → có thể gửi thêm 4 phím cùng lúc (không dùng ở đây)
```

### L2CAP PSM là gì?
PSM (Protocol/Service Multiplexer) giống như "port" trong TCP/IP:
- **PSM 17 (0x11)** — HID Control: quản lý phiên kết nối
- **PSM 19 (0x13)** — HID Interrupt: truyền dữ liệu phím thật sự (độ trễ thấp)

### Tại sao dùng `libc` thay vì thư viện cao cấp hơn?
Để kiểm soát tối đa độ trễ và bộ nhớ. Mỗi lần gõ phím chỉ là một lần gọi `send()` với 8 byte — không có overhead nào thêm.
