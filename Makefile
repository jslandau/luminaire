.PHONY: build dev test install-user uninstall-user

ifeq ($(shell uname -s),Darwin)
TAURI_BUILD_FLAGS := --bundles app
else
TAURI_BUILD_FLAGS :=
endif

build:
	cargo tauri build $(TAURI_BUILD_FLAGS)

dev:
	cargo tauri dev

test:
	cd src-tauri && cargo test

install-user:
	./scripts/install-user.sh

uninstall-user:
	./scripts/uninstall-user.sh
