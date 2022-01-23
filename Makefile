run: 
	cargo run
deploy:
	cargo build --target=arm-unknown-linux-musleabi 
	scp ./target/arm-unknown-linux-musleabi/debug/firmware pi@raspberrypi.local:~/