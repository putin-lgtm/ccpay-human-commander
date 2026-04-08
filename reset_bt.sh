#!/bin/bash
echo "🚀 Đang khởi tạo quy trình Hard-Reset Bluetooth (V3 - Anti-Timeout)..."

# 1. Giải phóng tài nguyên tầng ứng dụng
sudo killall -9 bluetoothd 2>/dev/null
sudo systemctl stop bluetooth

# 2. PHÁ BĂNG CHIP (Quan trọng nhất)
echo "⚡ Đang reset Kernel Driver..."
sudo modprobe -r btusb 2>/dev/null
sleep 1
sudo modprobe btusb
sleep 2 

# 3. Tự động tìm đường dẫn bluetoothd
BT_PATH=$(command -v bluetoothd)
if [ -z "$BT_PATH" ]; then BT_PATH="/usr/lib/bluetooth/bluetoothd"; fi

# 4. Sửa file Service
sudo sed -i "s|^ExecStart=.*|ExecStart=$BT_PATH -E --noplugin=input,hog|" /lib/systemd/system/bluetooth.service
sudo systemctl daemon-reload

# 5. Thiết lập Interface & Class
sudo hciconfig hci0 up
sleep 2 # Chờ chip ổn định
echo "📝 Đang nạp Class 0x0005C0..."
sudo hciconfig hci0 class 0x0005C0 || echo "❌ Vẫn bị timeout, hãy rút/cắm lại USB Bluetooth nếu có thể."

# 6. Khởi động lại Service
sudo systemctl start bluetooth
sleep 1

echo "✅ Trạng thái Class hiện tại:"
hciconfig hci0 class