# TidyFiles

Organizador de archivos para terminal, escrito en Rust. Clasifica los archivos sueltos de una carpeta en subdirectorios por tipo, con vista previa, interfaz interactiva y posibilidad de deshacer.

## Características

- Interfaz TUI construida con `ratatui` y `crossterm`.
- Configuración global en `~/.config/tidyfiles/config.toml` y configuración local opcional por carpeta.
- Categorías y extensiones personalizables.
- Resolución automática de colisiones mediante nombres como `archivo (1).ext`.
- Historial de transacciones y comando para deshacer.
- Modo CLI, simulación con `--dry-run` y soporte para archivos ocultos o desconocidos.
- Movimiento seguro entre distintos sistemas de archivos.

## Instalación

### Linux y macOS

```bash
curl -fsSL https://raw.githubusercontent.com/Echxvx2610/TidyFiles/main/install.sh | sh
```

El script descarga la última release, verifica su suma SHA256 y instala el binario en `~/.local/bin`.

Variables opcionales:

```bash
TIDYFILES_INSTALL_DIR=/usr/local/bin \
TIDYFILES_VERSION=v0.2.0 \
  curl -fsSL https://raw.githubusercontent.com/Echxvx2610/TidyFiles/main/install.sh | sh
```

### Desde el código

```bash
cargo build --release
```

El binario queda en `target/release/tidyfiles`. Los binarios de cada release también están disponibles en GitHub Releases.

## Uso

### Interfaz interactiva

```bash
tidyfiles <carpeta>
```

Si la entrada y la salida son una terminal, se abre la TUI. Para forzar el modo CLI:

```bash
tidyfiles <carpeta> --cli
```

### Controles de la TUI

| Tecla | Acción |
| :--- | :--- |
| `Flechas` o `k` / `j` | Navegar por la lista |
| `Espacio` | Seleccionar o deseleccionar un archivo |
| `a` | Seleccionar o deseleccionar todos |
| `Enter` u `o` | Organizar los archivos seleccionados |
| `u` | Deshacer la última organización |
| `c` | Ver categorías y formatos activos |
| `r` | Recargar el directorio |
| `q` o `Esc` | Salir |

### Modo CLI

```bash
# Simular sin mover archivos
tidyfiles <carpeta> --dry-run

# Organizar directamente
tidyfiles <carpeta> --cli

# Deshacer la última organización
tidyfiles <carpeta> --undo
```

### Opciones

| Opción | Descripción |
| :--- | :--- |
| `--tui` | Forzar la interfaz interactiva |
| `--cli` | Forzar el modo CLI |
| `-n`, `--dry-run` | Mostrar los movimientos sin ejecutarlos |
| `-u`, `--undo` | Revertir la última organización |
| `--hidden` | Incluir archivos ocultos |
| `--include-unknown` | Mover extensiones desconocidas a `Otros` |
| `-v`, `--verbose` | Mostrar el detalle de cada archivo |
| `--init-config` | Crear la configuración global si no existe |
| `--init-local` | Crear `tidyfiles.toml` en la carpeta actual |
| `--edit-config` | Abrir la configuración activa en el editor |
| `--config-path` | Mostrar la configuración que se está usando |

`--hidden` y `--include-unknown` solo aplican al modo CLI; la TUI siempre omite archivos ocultos y desconocidos.

## Configuración

TidyFiles busca la configuración en este orden:

1. `<carpeta>/tidyfiles.toml`
2. `<carpeta>/.tidyfiles.toml`
3. `~/.config/tidyfiles/config.toml`
4. Reglas predeterminadas en memoria

La configuración local solo afecta a esa carpeta. Si no existe configuración local ni global, la global se crea automáticamente con las reglas predeterminadas.

```bash
tidyfiles --config-path   # Ruta y ámbito de la configuración activa
tidyfiles --init-config   # Crear la configuración global
tidyfiles --init-local    # Crear una configuración local
tidyfiles --edit-config   # Editar la configuración activa
```

Ejemplo de configuración:

```toml
[categories.Imagenes]
icon = "IMG"
extensions = ["jpg", "jpeg", "png", "gif", "webp", "svg"]

[categories.Documentos]
icon = "DOC"
extensions = ["pdf", "docx", "txt", "md", "csv"]

[categories.Modelos3D]
icon = "3D"
extensions = ["blend", "fbx", "obj", "stl"]
```

El campo `icon` acepta cualquier texto. La clave de cada categoría se usa como nombre de la carpeta de destino.

## Pruebas

```bash
cargo test
```

Las pruebas de integración cubren clasificación, resolución de colisiones, configuración personalizada y el ciclo completo de organizar y deshacer.
