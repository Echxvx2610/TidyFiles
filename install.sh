#!/bin/sh

# Instalador de TidyFiles para Linux y macOS.
# Variables opcionales:
#   TIDYFILES_VERSION      Version a instalar, con o sin prefijo "v".
#   TIDYFILES_INSTALL_DIR  Directorio de instalación (por defecto ~/.local/bin).

set -eu

REPO="Echxvx2610/TidyFiles"
BIN="tidyfiles"
INSTALL_DIR="${TIDYFILES_INSTALL_DIR:-${HOME}/.local/bin}"

fail() {
    printf 'Error: %s\n' "$*" >&2
    exit 1
}

need() {
    command -v "$1" >/dev/null 2>&1 || fail "Se requiere '$1' para continuar."
}

need curl
need tar
need uname
need find

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-unknown-linux-musl"
                ;;
            aarch64|arm64)
                TARGET="aarch64-unknown-linux-gnu"
                ;;
            *)
                fail "Arquitectura Linux no soportada: ${ARCH}."
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64|aarch64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                fail "Arquitectura macOS no soportada: ${ARCH}."
                ;;
        esac
        ;;
    *)
        fail "Sistema no soportado: ${OS}. Descarga un binario desde GitHub Releases."
        ;;
esac

if [ -n "${TIDYFILES_VERSION:-}" ]; then
    TAG="${TIDYFILES_VERSION}"
    case "$TAG" in
        v*) ;;
        *) TAG="v${TAG}" ;;
    esac
else
    RELEASE_JSON="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")" \
        || fail "No se pudo consultar la última release de ${REPO}."
    TAG="$(printf '%s\n' "$RELEASE_JSON" \
        | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \
        | head -n 1)"
    [ -n "$TAG" ] || fail "No se encontró una release publicada. Define TIDYFILES_VERSION."
fi

ASSET_BASE="${BIN}-${TAG}-${TARGET}"
ASSET="${ASSET_BASE}.tar.gz"
CHECKSUM="${ASSET_BASE}.sha256"
BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"
TMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/tidyfiles.XXXXXX")"

cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup 0 1 2 15

printf 'Descargando %s...\n' "$ASSET"
curl -fsSL --retry 3 -o "${TMP_DIR}/${ASSET}" "${BASE_URL}/${ASSET}" \
    || fail "No se pudo descargar ${ASSET}."
curl -fsSL --retry 3 -o "${TMP_DIR}/${CHECKSUM}" "${BASE_URL}/${CHECKSUM}" \
    || fail "No se pudo descargar la suma SHA256 de ${ASSET}."

if command -v sha256sum >/dev/null 2>&1; then
    (cd "$TMP_DIR" && sha256sum -c "${CHECKSUM}") >/dev/null \
        || fail "La verificación SHA256 falló."
elif command -v shasum >/dev/null 2>&1; then
    (cd "$TMP_DIR" && shasum -a 256 -c "${CHECKSUM}") >/dev/null \
        || fail "La verificación SHA256 falló."
else
    fail "Se requiere 'sha256sum' o 'shasum' para verificar la descarga."
fi

mkdir -p "${TMP_DIR}/extract"
tar -xzf "${TMP_DIR}/${ASSET}" -C "${TMP_DIR}/extract"
BIN_PATH="$(find "${TMP_DIR}/extract" -type f -name "$BIN" | head -n 1)"
[ -n "$BIN_PATH" ] || fail "El archivo descargado no contiene el binario ${BIN}."

mkdir -p "$INSTALL_DIR"
cp "$BIN_PATH" "${INSTALL_DIR}/${BIN}"
chmod 755 "${INSTALL_DIR}/${BIN}"

printf 'TidyFiles %s instalado en %s\n' "$TAG" "${INSTALL_DIR}/${BIN}"

case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        printf 'Añade %s a PATH para ejecutar %s desde cualquier directorio.\n' \
            "$INSTALL_DIR" "$BIN"
        ;;
esac
