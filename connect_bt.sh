#!/bin/bash
MAC="F4:7D:EF:8A:3B:5C"

echo "🔗 Đang chuẩn bị kết nối Bypass xác thực..."

# Sử dụng bluetoothctl với Agent tự động chấp nhận
# 'agent NoInputNoOutput' sẽ ép hệ thống không yêu cầu xác nhận PIN
# 'default-agent' làm cho agent này trở thành mặc định cho mọi giao dịch
sudo bluetoothctl << EOF
power on
agent NoInputNoOutput
default-agent
remove $MAC
scan on
sleep 5
pair $MAC
trust $MAC
connect $MAC
quit
EOF

echo "✅ Đã gửi lệnh kết nối Bypass. Kiểm tra điện thoại!"