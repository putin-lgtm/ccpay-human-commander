#!/bin/bash
MAC="F4:7D:EF:8A:3B:5C"

echo "🔗 Đang chuẩn bị kết nối Bypass xác thực..."

# Chạy bluetoothctl theo từng dòng để đảm bảo 'sleep' nằm ở ngoài Bash
bluetoothctl power on
# Tắt agent cũ nếu có để tránh lỗi "Failed to register"
bluetoothctl agent off 
bluetoothctl agent NoInputNoOutput
bluetoothctl default-agent
bluetoothctl remove $MAC

echo "📡 Đang quét thiết bị..."
# Quét ngầm trong 5 giây rồi tắt
timeout 5s bluetoothctl scan on > /dev/null

echo "🤝 Đang tiến hành Pair & Trust..."
bluetoothctl pair $MAC
bluetoothctl trust $MAC

echo "🔌 Đang kết nối..."
bluetoothctl connect $MAC

echo "✅ Đã gửi lệnh kết nối. Hãy kiểm tra màn hình điện thoại!"