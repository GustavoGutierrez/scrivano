#!/bin/bash
set -e

echo "=== MeetWhisperer Build Script ==="

# Configurar rutas de librerías
export LIBCLANG_PATH=/tmp/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
export LD_LIBRARY_PATH=/home/meridian/lib:$LD_LIBRARY_PATH

# Verificar que LLVM esté disponible
if [ ! -d "$LIBCLANG_PATH" ]; then
    echo "Descargando LLVM 14..."
    cd /tmp
    wget -q https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.0/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04.tar.xz -O clang.tar.xz
    tar -xf clang.tar.xz
    export LIBCLANG_PATH=/tmp/clang+llvm-14.0.0-x86_64-linux-gnu-ubuntu-18.04/lib
fi

# Verificar modelo Whisper
if [ ! -f "models/ggml-small.bin" ]; then
    echo "Descargando modelo Whisper ggml-small.bin..."
    mkdir -p models
    wget -P models/ https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin
fi

echo "=== Compilando en modo debug ==="
cargo build

echo "=== Compilando en modo release ==="
cargo build --release

echo "=== Ejecutando tests ==="
cargo test

echo "=== Verificando formato ==="
cargo fmt --check

echo "=== Verificando con clippy ==="
cargo clippy --all-targets

echo "=== Creando distribución ==="
rm -rf dist
mkdir -p dist/meet-whisperer

# Copiar ejecutable
cp target/release/meet-whisperer dist/meet-whisperer/

# Copiar modelo
cp -r models dist/meet-whisperer/

# Copiar librerías necesarias
echo "Copiando librerías..."
ldd target/release/meet-whisperer | grep -v "linux-vdso\|not found" | awk '{print $3}' | xargs -I{} cp -v {} dist/meet-whisperer/ 2>/dev/null || true

# Crear README de distribución
cat > dist/meet-whisperer/README.txt << 'EOF'
# MeetWhisperer - Distribución

## Requisitos del sistema
- Linux con PulseAudio o PipeWire
- Bibliotecas GTK3

## Ejecución
cd meet-whisperer
./meet-whisperer

## Uso
1. Ejecutar la aplicación
2. "Iniciar grabación" para capturar audio del sistema
3. "Detener grabación" para transcribir
EOF

# Crear archivo tar
cd dist
tar -czvf meet-whisperer.tar.gz meet-whisperer/

echo "=== Distribución creada ==="
echo "Ejecutable: target/release/meet-whisperer"
echo "Distribución: dist/meet-whisperer.tar.gz"
ls -lh meet-whisperer.tar.gz
