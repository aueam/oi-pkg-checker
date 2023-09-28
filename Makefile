build: build_release download_catalog download_encumbered_catalog

build_release:
	cargo build --release

download_catalog:
	curl -o assets/catalog.dependency.C https://pkg.openindiana.org/hipster/catalog/1/catalog.dependency.C

download_encumbered_catalog:
	curl -o assets/catalog.encumbered.dependency.C https://pkg.openindiana.org/hipster-encumbered/catalog/1/catalog.dependency.C

clean:
	rm assets/catalog.dependency.C assets/catalog.encumbered.dependency.C
	rm oi-pkg-checker
	cargo clean
