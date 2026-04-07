# Makefile — Scrivano
# Targets: build, release, deb, dist, install, uninstall, clean

PKG_NAME    := scrivano
VERSION     := 0.1.0
ARCH        := amd64
BINARY      := target/release/$(PKG_NAME)
DEB_FILE    := packaging/$(PKG_NAME)_$(VERSION)_$(ARCH).deb
INSTALL_DIR := /opt/$(PKG_NAME)

export LIBCLANG_PATH := /tmp/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
export LD_LIBRARY_PATH := /home/meridian/lib:$(LD_LIBRARY_PATH)

.PHONY: all build release deb dist install uninstall clean help

# ── Default ───────────────────────────────────────────────────────────────────
all: release

help: ## Muestra esta ayuda
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ── Compilación ───────────────────────────────────────────────────────────────
build: ## Compilar en modo debug
	cargo build

release: llvm-check ## Compilar en modo release (optimizado)
	cargo build --release
	@echo "✓ Binario: $(BINARY) ($(shell du -sh $(BINARY) | cut -f1))"

# ── Empaquetado ───────────────────────────────────────────────────────────────
deb: release ## Generar paquete .deb (compila + empaca)
	@bash package-deb.sh --no-build

deb-only: ## Generar .deb sin recompilar
	@bash package-deb.sh --no-build

dist: release ## Generar tarball portable (.tar.gz)
	@bash dist.sh --no-build 2>/dev/null || bash dist.sh

# ── Instalación local (sin .deb) ──────────────────────────────────────────────
install: release ## Instalar en el sistema actual (requiere sudo)
	@echo "Instalando en $(INSTALL_DIR)..."
	sudo mkdir -p $(INSTALL_DIR)/lib $(INSTALL_DIR)/models
	sudo cp $(BINARY) $(INSTALL_DIR)/
	@if [ -d models ]; then sudo cp -r models/*.bin $(INSTALL_DIR)/models/ 2>/dev/null || true; fi
	@ldd $(BINARY) | grep -v "linux-vdso\|not found\|ld-linux" | awk '{print $$3}' | grep -v '^$$' \
		| xargs -I{} sudo cp -n {} $(INSTALL_DIR)/lib/ 2>/dev/null || true
	@printf '#!/bin/bash\nDIR="$(INSTALL_DIR)"\ncd "$$DIR"\nexport LD_LIBRARY_PATH="$$DIR/lib:$${LD_LIBRARY_PATH:-}"\nexec "$$DIR/$(PKG_NAME)" "$$@" 2> >(grep -Ev "Gtk-CRITICAL|GLib-GObject|whisper_model_load|whisper_init" >&2)\n' \
		| sudo tee $(INSTALL_DIR)/run.sh > /dev/null
	sudo chmod 755 $(INSTALL_DIR)/run.sh $(INSTALL_DIR)/$(PKG_NAME)
	sudo ln -sf $(INSTALL_DIR)/run.sh /usr/local/bin/$(PKG_NAME)
	@# Iconos
	@for size in 16 32 64 128 256 512; do \
		src="assets/favicons/favicon-$${size}x$${size}.png"; \
		dst="/usr/share/icons/hicolor/$${size}x$${size}/apps/$(PKG_NAME).png"; \
		[ -f "$$src" ] && sudo cp "$$src" "$$dst" || true; \
	done
	@# .desktop
	@printf '[Desktop Entry]\nType=Application\nName=Scrivano\nExec=$(INSTALL_DIR)/run.sh\nIcon=$(PKG_NAME)\nTerminal=false\nCategories=AudioVideo;Utility;\n' \
		| sudo tee /usr/share/applications/$(PKG_NAME).desktop > /dev/null
	sudo update-icon-caches /usr/share/icons/hicolor 2>/dev/null || true
	sudo update-desktop-database /usr/share/applications 2>/dev/null || true
	@echo "✓ Instalado. Ejecuta: $(PKG_NAME)"

uninstall: ## Desinstalar del sistema actual (requiere sudo)
	sudo rm -rf $(INSTALL_DIR)
	sudo rm -f /usr/local/bin/$(PKG_NAME)
	sudo rm -f /usr/share/applications/$(PKG_NAME).desktop
	@for size in 16 32 64 128 256 512; do \
		sudo rm -f "/usr/share/icons/hicolor/$${size}x$${size}/apps/$(PKG_NAME).png"; \
	done
	sudo update-icon-caches /usr/share/icons/hicolor 2>/dev/null || true
	sudo update-desktop-database /usr/share/applications 2>/dev/null || true
	@echo "✓ Desinstalado"

# ── Limpieza ──────────────────────────────────────────────────────────────────
clean: ## Eliminar artefactos de compilación y empaquetado
	cargo clean
	rm -rf dist/ packaging/
	@echo "✓ Limpieza completa"

clean-pkg: ## Eliminar solo los artefactos de empaquetado
	rm -rf dist/ packaging/

# ── Utilidades ────────────────────────────────────────────────────────────────
llvm-check: ## Verificar/descargar LLVM (requerido para whisper-rs)
	@if [ ! -d "$(LIBCLANG_PATH)" ]; then \
		echo "[make] Descargando LLVM 14..."; \
		cd /tmp && \
		wget -q https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz -O clang.tar.xz && \
		tar -xf clang.tar.xz; \
	fi

check: ## Verificar el paquete .deb generado
	@[ -f "$(DEB_FILE)" ] || { echo "ERROR: $(DEB_FILE) no existe. Ejecuta: make deb"; exit 1; }
	@echo "=== Información del paquete ==="
	dpkg-deb --info $(DEB_FILE)
	@echo ""
	@echo "=== Contenido ==="
	dpkg-deb --contents $(DEB_FILE)
