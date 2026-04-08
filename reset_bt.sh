#!/bin/bash
echo "🚀 Đang khởi tạo quy trình Hard-Reset Bluetooth..."

# 1. Giết các tiến trình cũ để tránh tranh chấp socket
sudo killall -9 bluetoothd 2>/dev/null
sudo systemctl stop bluetooth

# 2. Kiểm tra đường dẫn thực thi của bluetoothd
if [ -f "/usr/libexec/bluetooth/bluetoothd" ]; then
    BT_PATH="/usr/libexec/bluetooth/bluetoothd"
else
    BT_PATH="/usr/lib/bluetooth/bluetoothd"
fi

# 3. Sửa file Service tự động (Dùng sed để tránh lỗi tay)
sudo sed -i "s|^ExecStart=.*|ExecStart=$BT_PATH -E --noplugin=input,hog|" /lib/systemd/system/bluetooth.service

# 4. Nạp lại hệ thống
sudo systemctl daemon-reload
sudo systemctl start bluetooth
sleep 2

# 5. Cấu hình Class Combo (Keyboard + Mouse)
sudo hciconfig hci0 up
sudo hciconfig hci0 class 0x0005C0

echo "✅ Hệ thống đã sẵn sàng với Class: $(hciconfig hci0 class | grep Class)"
echo "👉 Bây giờ hãy chạy App Rust, sau đó mới dùng bluetoothctl để connect."