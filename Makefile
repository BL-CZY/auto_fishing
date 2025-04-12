build: 
	cargo build --release

install: build
	mkdir -p ~/.cache/auto_fishing/
	rm -rf ~/.local/share/auto_fishing/
	sudo cp ./target/release/auto_fishing /usr/bin/auto_fishing
