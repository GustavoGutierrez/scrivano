#!/bin/bash
# dist.sh — Genera la distribución portable de Scrivano
# Uso: ./dist.sh [--release-only]
set -e

# ── Configuración ─────────────────────────────────────────────────────────────
export LIBCLANG_PATH=/tmp/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
export LD_LIBRARY_PATH=/home/meridian/lib:$LD_LIBRARY_PATH

APP_NAME="scrivano"
DIST_DIR="dist/$APP_NAME"
BINARY="target/release/$APP_NAME"

# ── Paso 1: Verificar LLVM ────────────────────────────────────────────────────
if [ ! -d "$LIBCLANG_PATH" ]; then
    echo "[dist] Descargando LLVM 14 (requerido para compilar whisper-rs)..."
    cd /tmp
    wget -q https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz -O clang.tar.xz
    tar -xf clang.tar.xz
    cd -
fi

# ── Paso 2: Compilar en release ───────────────────────────────────────────────
echo "[dist] Compilando en modo release..."
cargo build --release
echo "[dist] Compilación exitosa: $BINARY"

# ── Paso 3: Empacar distribución ──────────────────────────────────────────────
echo "[dist] Empacando distribución..."
rm -rf dist
mkdir -p "$DIST_DIR"

# Ejecutable
cp "$BINARY" "$DIST_DIR/"

# Modelos Whisper (obligatorios para que la app funcione)
if [ -d "models" ]; then
    cp -r models "$DIST_DIR/"
    echo "[dist] Modelos incluidos: $(ls models/*.bin 2>/dev/null | wc -l)"
else
    echo "[ADVERTENCIA] No se encontró la carpeta models/ — la app no podrá transcribir sin un modelo."
    mkdir -p "$DIST_DIR/models"
fi

# Librerías compartidas (portabilidad en otras distros Linux)
echo "[dist] Copiando librerías compartidas..."
ldd "$BINARY" \
    | grep -v "linux-vdso\|not found" \
    | awk '{print $3}' \
    | grep -v '^$' \
    | xargs -I{} cp -n {} "$DIST_DIR/" 2>/dev/null || true

# Script de lanzamiento con LD_LIBRARY_PATH apuntando a las libs empacadas
cat > "$DIST_DIR/run.sh" << 'EOF'
#!/bin/bash
# Lanzador portable — ajusta LD_LIBRARY_PATH para usar las libs incluidas
DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="$DIR:$LD_LIBRARY_PATH"
exec "$DIR/scrivano" "$@"
EOF
chmod +x "$DIST_DIR/run.sh"

# ── Paso 4: Crear tarball ─────────────────────────────────────────────────────
cd dist
tar -czf "$APP_NAME.tar.gz" "$APP_NAME/"
cd ..

echo ""
echo "=== Distribución lista ==="
echo "  Directorio : dist/$APP_NAME/"
echo "  Tarball    : dist/$APP_NAME.tar.gz"
echo "  Tamaño     : $(du -sh dist/$APP_NAME.tar.gz | cut -f1)"
echo ""
echo "  Para ejecutar desde el tarball:"
echo "    tar -xzf dist/$APP_NAME.tar.gz"
echo "    ./$APP_NAME/run.sh"
