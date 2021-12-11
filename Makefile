default: build_and_send

build_and_send: build send

build:
	cargo build --target=aarch64-unknown-linux-gnu

send:
	scp target/aarch64-unknown-linux-gnu/debug/waterpi pi@waterpi:~/waterpi/waterpi

