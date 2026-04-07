#!/bin/bash
# package-deb.sh — Genera el paquete .deb de Scrivano para Ubuntu/Debian
#
# Uso:
#   ./package-deb.sh            # compila + empaca
#   ./package-deb.sh --no-build # sólo empaca (asume que target/release ya existe)
#
# Requiere: dpkg-deb, cargo, LIBCLANG_PATH configurado
set -euo pipefail

# ── Configuración ─────────────────────────────────────────────────────────────
PKG_NAME="scrivano"
PKG_VERSION="0.1.0"
PKG_ARCH="amd64"
PKG_MAINTAINER="Gustavo Gutiérrez <gustavo@example.com>"
PKG_DESCRIPTION="Transcripción local de reuniones con Whisper AI"
PKG_LONG_DESC="Graba el audio del sistema y lo transcribe offline usando modelos Whisper.
 Integración opcional con Ollama para mejorar la transcripción con LLMs locales.
 100% offline — ningún dato sale del equipo."

BINARY="target/release/${PKG_NAME}"
DEB_ROOT="packaging/${PKG_NAME}_${PKG_VERSION}_${PKG_ARCH}"
INSTALL_DIR="/opt/${PKG_NAME}"
ASSETS_DIR="assets"

export LIBCLANG_PATH=/tmp/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
export LD_LIBRARY_PATH=/home/meridian/lib:${LD_LIBRARY_PATH:-}

# ── Colores ───────────────────────────────────────────────────────────────────
GREEN='\033[0;32m'; CYAN='\033[0;36m'; YELLOW='\033[1;33m'; NC='\033[0m'
step() { echo -e "${CYAN}[deb]${NC} $1"; }
ok()   { echo -e "${GREEN}[ok]${NC}  $1"; }
warn() { echo -e "${YELLOW}[!]${NC}   $1"; }

# ── Paso 0: Verificar herramientas ────────────────────────────────────────────
step "Verificando herramientas necesarias..."
for tool in dpkg-deb cargo; do
    command -v "$tool" >/dev/null 2>&1 || { echo "ERROR: '$tool' no encontrado. Instala con: sudo apt install dpkg"; exit 1; }
done

# ── Paso 1: Compilar (opcional) ───────────────────────────────────────────────
if [[ "${1:-}" != "--no-build" ]]; then
    step "Compilando en modo release..."
    if [ ! -d "$LIBCLANG_PATH" ]; then
        warn "LLVM no encontrado en $LIBCLANG_PATH — descargando..."
        cd /tmp
        wget -q https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz -O clang.tar.xz
        tar -xf clang.tar.xz
        cd -
    fi
    cargo build --release
    ok "Compilación exitosa"
else
    step "Saltando compilación (--no-build)"
    [ -f "$BINARY" ] || { echo "ERROR: $BINARY no existe. Compila primero."; exit 1; }
fi

# ── Paso 2: Limpiar y crear estructura del paquete ────────────────────────────
step "Creando estructura del paquete .deb..."
rm -rf "$DEB_ROOT"

# Directorios del sistema de archivos destino
mkdir -p "${DEB_ROOT}/DEBIAN"
mkdir -p "${DEB_ROOT}${INSTALL_DIR}/lib"
mkdir -p "${DEB_ROOT}${INSTALL_DIR}/models"
mkdir -p "${DEB_ROOT}/usr/bin"
mkdir -p "${DEB_ROOT}/usr/share/applications"
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/16x16/apps"
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/32x32/apps"
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/64x64/apps"
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/128x128/apps"
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/256x256/apps"
mkdir -p "${DEB_ROOT}/usr/share/icons/hicolor/512x512/apps"

# ── Paso 3: Copiar binario ────────────────────────────────────────────────────
step "Copiando binario..."
cp "$BINARY" "${DEB_ROOT}${INSTALL_DIR}/"
chmod 755 "${DEB_ROOT}${INSTALL_DIR}/${PKG_NAME}"

# ── Paso 4: Copiar librerías compartidas ──────────────────────────────────────
step "Copiando librerías compartidas..."
ldd "$BINARY" \
    | grep -v "linux-vdso\|not found\|ld-linux" \
    | awk '{print $3}' \
    | grep -v '^$' \
    | while read -r lib; do
        cp -n "$lib" "${DEB_ROOT}${INSTALL_DIR}/lib/" 2>/dev/null && echo "  → $(basename "$lib")" || true
      done

# ── Paso 5: Copiar modelos Whisper ────────────────────────────────────────────
step "Copiando modelos Whisper..."
if [ -d "models" ] && [ "$(ls models/*.bin 2>/dev/null | wc -l)" -gt 0 ]; then
    cp models/*.bin "${DEB_ROOT}${INSTALL_DIR}/models/"
    ok "Modelos incluidos: $(ls models/*.bin | wc -l)"
else
    warn "No se encontraron modelos .bin en models/ — el usuario deberá descargarlos."
fi

# ── Paso 6: Script lanzador ───────────────────────────────────────────────────
step "Creando script lanzador..."
cat > "${DEB_ROOT}${INSTALL_DIR}/run.sh" << 'LAUNCHER'
#!/bin/bash
DIR="/opt/scrivano"
cd "$DIR"                                      # modelos se buscan relativos al cwd
export LD_LIBRARY_PATH="$DIR/lib:${LD_LIBRARY_PATH:-}"

# Silenciar warnings de GTK/GLib del tray icon (benignos en algunos DEs)
# y mensajes internos de whisper.cpp — solo mostrar errores reales de la app.
exec "$DIR/scrivano" "$@" \
    2> >(grep -Ev \
        "Gtk-CRITICAL|GLib-GObject|GLib-GLib|whisper_model_load|whisper_init_with_params|whisper_init_from_file" \
        >&2)
LAUNCHER
chmod 755 "${DEB_ROOT}${INSTALL_DIR}/run.sh"

# Symlink en /usr/bin para llamarlo desde terminal
ln -sf "${INSTALL_DIR}/run.sh" "${DEB_ROOT}/usr/bin/${PKG_NAME}"

# ── Paso 7: Iconos ────────────────────────────────────────────────────────────
step "Copiando iconos..."
FAVICON_DIR="${ASSETS_DIR}/favicons"
for size in 16 32 64 128 256 512; do
    src="${FAVICON_DIR}/favicon-${size}x${size}.png"
    if [ -f "$src" ]; then
        cp "$src" "${DEB_ROOT}/usr/share/icons/hicolor/${size}x${size}/apps/${PKG_NAME}.png"
    fi
done

# ── Paso 8: Archivo .desktop (lanzador de escritorio) ────────────────────────
step "Creando entrada .desktop..."
cat > "${DEB_ROOT}/usr/share/applications/${PKG_NAME}.desktop" << DESKTOP
[Desktop Entry]
Version=1.0
Type=Application
Name=Scrivano
GenericName=Transcriptor de reuniones
Comment=Transcripción local de audio del sistema usando Whisper AI
Exec=${INSTALL_DIR}/run.sh
Icon=${PKG_NAME}
Terminal=false
Categories=AudioVideo;Audio;Utility;
Keywords=transcription;whisper;meeting;audio;ai;ollama;
StartupWMClass=scrivano
DESKTOP

# ── Paso 9: Calcular tamaño instalado ─────────────────────────────────────────
INSTALLED_SIZE=$(du -sk "${DEB_ROOT}" | cut -f1)

# ── Paso 10: Archivo DEBIAN/control ──────────────────────────────────────────
step "Generando DEBIAN/control..."
cat > "${DEB_ROOT}/DEBIAN/control" << CONTROL
Package: ${PKG_NAME}
Version: ${PKG_VERSION}
Section: sound
Priority: optional
Architecture: ${PKG_ARCH}
Installed-Size: ${INSTALLED_SIZE}
Depends: libpulse0 (>= 8.0), libglib2.0-0 (>= 2.50)
Recommends: ollama
Maintainer: ${PKG_MAINTAINER}
Description: ${PKG_DESCRIPTION}
$(echo "${PKG_LONG_DESC}" | sed 's/^/ /')
Homepage: https://github.com/gustavo/scrivano
CONTROL

# ── Paso 11: Scripts de mantenimiento ─────────────────────────────────────────
# postinst: se ejecuta después de instalar
cat > "${DEB_ROOT}/DEBIAN/postinst" << 'POSTINST'
#!/bin/bash
set -e
# Actualizar caché de íconos y base de datos de aplicaciones
if command -v update-icon-caches >/dev/null 2>&1; then
    update-icon-caches /usr/share/icons/hicolor || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi
POSTINST
chmod 755 "${DEB_ROOT}/DEBIAN/postinst"

# prerm: se ejecuta antes de desinstalar
cat > "${DEB_ROOT}/DEBIAN/prerm" << 'PRERM'
#!/bin/bash
set -e
# Nada que hacer antes de remover
exit 0
PRERM
chmod 755 "${DEB_ROOT}/DEBIAN/prerm"

# postrm: se ejecuta después de desinstalar
cat > "${DEB_ROOT}/DEBIAN/postrm" << 'POSTRM'
#!/bin/bash
set -e
# Limpiar base de datos del usuario (opcional, solo si purge)
if [ "$1" = "purge" ]; then
    rm -rf /opt/scrivano || true
fi
if command -v update-icon-caches >/dev/null 2>&1; then
    update-icon-caches /usr/share/icons/hicolor || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database /usr/share/applications || true
fi
POSTRM
chmod 755 "${DEB_ROOT}/DEBIAN/postrm"

# ── Paso 12: Permisos correctos ───────────────────────────────────────────────
step "Ajustando permisos..."
# Directorios
find "${DEB_ROOT}" -type d -exec chmod 755 {} \;
# Archivos regulares: 644 por defecto
find "${DEB_ROOT}" -type f \
    ! -path "*/DEBIAN/postinst" \
    ! -path "*/DEBIAN/prerm" \
    ! -path "*/DEBIAN/postrm" \
    -exec chmod 644 {} \;
# Ejecutables
chmod 755 "${DEB_ROOT}${INSTALL_DIR}/${PKG_NAME}"
chmod 755 "${DEB_ROOT}${INSTALL_DIR}/run.sh"
# Nota: el symlink en usr/bin NO necesita chmod (los symlinks son siempre lrwxrwxrwx)

# ── Paso 13: Generar md5sums ──────────────────────────────────────────────────
step "Calculando checksums..."
cd "${DEB_ROOT}"
find . -type f ! -path './DEBIAN/*' -exec md5sum {} \; > DEBIAN/md5sums
cd -

# ── Paso 14: Construir el .deb ────────────────────────────────────────────────
step "Construyendo paquete .deb (puede tardar ~1 min por la compresión)..."
mkdir -p packaging
DEB_FILE="packaging/${PKG_NAME}_${PKG_VERSION}_${PKG_ARCH}.deb"
# fakeroot asigna root/root a los archivos sin requerir privilegios reales
fakeroot dpkg-deb --build "${DEB_ROOT}" "$DEB_FILE"

echo ""
echo -e "${GREEN}=== Paquete .deb listo ===${NC}"
echo "  Archivo : $DEB_FILE"
echo "  Tamaño  : $(du -sh "$DEB_FILE" | cut -f1)"
echo ""
echo "  Para instalar:"
echo "    sudo dpkg -i $DEB_FILE"
echo "    sudo apt-get install -f   # si hay dependencias faltantes"
echo ""
echo "  Para verificar el contenido:"
echo "    dpkg-deb --contents $DEB_FILE"
