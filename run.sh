sudo ip link set can0 up type can bitrate 500000
sudo ifconfig can0 txqueuelen 65536
./target/release/zenoh-angle &
./target/release/zenoh-server &
./target/release/zenoh-finger left ./model.pth 3 &
