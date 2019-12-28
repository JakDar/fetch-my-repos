build:
	cargo build --release

install_sym: build
	mkdir -p ~/.local/bin
	ln -sf $$(pwd)/target/release/gclone-fetch ~/.local/bin/gclone-fetch
	ln -sf $$(pwd)/gclone.sh ~/.local/bin/gclone
	gclone --install

install:
	mkdir -p ~/.local/bin
	cp $$(pwd)/target/release/gclone-fetch ~/.local/bin/gclone-fetch
	cp $$(pwd)/gclone.sh ~/.local/bin/gclone
	gclone --install

uninstall:
	gclone --uninstall
	rm ~/.local/bin/gclone-fetch
	rm ~/.local/bin/gclone
