#!/bin/bash

echo "🧹 Đang dọn dẹp Stack Bluetooth..."
# 1. Dừng service mặc định để giải phóng UUID
sudo systemctl stop bluetooth

# 2. Kill các tiến trình bluetoothd còn sót lại (nếu có)
sudo killall bluetoothd 2>/dev/null

# 3. Chạy bluetoothd ở chế độ "Sạch" (Exclude các plugin tranh chấp)
# Chúng ta dùng '&' để nó chạy ngầm, hoặc bạn chạy nó ở một Terminal riêng
sudo /usr/libexec/bluetooth/bluetoothd -n -E --noplugin=input,hog > /dev/null 2>&1 &

sleep 2 # Đợi daemon khởi động

# 4. Cấu hình Controller
sudo btmgmt power on
sudo btmgmt pairable on
sudo btmgmt discov on

echo "✅ Bluetooth Clean & Ready (Plugins input/hog excluded)!"