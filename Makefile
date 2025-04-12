build: 
	cargo build --release

install: build
	mkdir -p ~/.cache/auto_fishing/
	rm -r ~/.local/share/auto_fishing/
	mkdir -p ~/.local/share/auto_fishing/
	cp ./assets/ ~/.local/share/auto_fishing/assets/
	sudo cp ./target/release/auto_fishing /usr/bin/auto_fishing
