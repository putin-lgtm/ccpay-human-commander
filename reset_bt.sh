#!/bin/bash
echo "🚀 Đang khởi tạo quy trình Hard-Reset Bluetooth (Hybrid Version)..."

# 1. Ép buộc giải phóng tài nguyên (Phong cách hệ thống cũ)
sudo killall -9 bluetoothd 2>/dev/null
sudo systemctl stop bluetooth

# 2. Tự động tìm đường dẫn (Bypass mọi phiên bản Ubuntu)
if [ -f "/usr/libexec/bluetooth/bluetoothd" ]; then
    BT_PATH="/usr/libexec/bluetooth/bluetoothd"
else
    BT_PATH="/usr/lib/bluetooth/bluetoothd"
fi

# 3. Cấu hình Service "Sạch" (Bypass tranh chấp HID)
sudo sed -i "s|^ExecStart=.*|ExecStart=$BT_PATH -E --noplugin=input,hog|" /lib/systemd/system/bluetooth.service

# 4. Kích hoạt lại Service
sudo systemctl daemon-reload
sudo systemctl start bluetooth
sleep 2

# 5. Cấu hình Class Combo (Kích hoạt Cursor cho Samsung)
# Bước này phải Up interface trước khi ghi Class
sudo hciconfig hci0 up
sleep 1
sudo hciconfig hci0 class 0x0005C0

echo "✅ Hệ thống đã sẵn sàng!"
hciconfig hci0 class | grep Class
echo "👉 Bước tiếp theo: Chạy App Rust -> Bluetoothctl Connect."