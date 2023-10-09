build: build_release download_repo download_catalog download_encumbered_catalog

update: download_catalog download_encumbered_catalog update_repo

build_release:
	cargo build --release

download_catalog:
	curl -o assets/catalog.dependency.C https://pkg.openindiana.org/hipster/catalog/1/catalog.dependency.C

download_encumbered_catalog:
	curl -o assets/catalog.encumbered.dependency.C https://pkg.openindiana.org/hipster-encumbered/catalog/1/catalog.dependency.C

download_repo:
	git clone https://github.com/OpenIndiana/oi-userland.git assets/oi-userland

update_repo:
	cd assets/oi-userland/ && git pull

clean:
	rm assets/catalog.dependency.C assets/catalog.encumbered.dependency.C
	rm oi-pkg-checker
	cargo clean
