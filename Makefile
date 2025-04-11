build: 
	cargo build --release

install: build
	sudo cp ./target/release/auto_fishing /usr/bin/auto_fishing
