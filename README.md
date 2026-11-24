# ⚡ TidyFiles

Un organizador de archivos ultrarrápido, visual y seguro escrito en **Rust**. Diseñado especialmente para carpetas saturadas de descargas y documentos desordenados.

---

## ✨ Características Principales

- 🖥️ **TUI Interactiva (Terminal User Interface)**: Construida con `ratatui` y `crossterm`. Permite previsualizar qué archivos se moverán, marcarlos o desmarcarlos con el teclado y confirmar en un solo clic.
- ⚙️ **Configuración Centralizada Única (`~/.config/tidyfiles/config.toml`)**:
  - Un único archivo global para todas las carpetas del sistema.
  - Añade nuevas categorías (ej. Modelos 3D, Torrents, Libros, etc.).
  - Configura tus propias extensiones e iconos emoji por categoría.
- 🛡️ **Seguridad contra colisiones**: Si ya existe un archivo con el mismo nombre en destino, renombra automáticamente a `archivo (1).ext` para no sobreescribir jamás tus datos.
- 🔄 **Sistema Undo / Deshacer**: Mantiene un historial seguro de las transacciones para restaurar los archivos a su posición original si cambias de opinión.
- 🚀 **Modo CLI / Scripting**: Soporte para scripts desatendidos, modo de simulación (`--dry-run`) y argumentos de consola directos.

---

## 🎮 Controles de la Interfaz TUI

Al ejecutar `tidyfiles <carpeta>` en tu terminal, se abrirá la interfaz interactiva con los siguientes controles:

| Tecla | Acción |
| :--- | :--- |
| `↑` / `↓` o `k` / `j` | Navegar por la lista de archivos detectados |
| `[Espacio]` | Seleccionar / Deseleccionar el archivo resaltado |
| `[a]` | Seleccionar o deseleccionar todos los archivos |
| `[Enter]` o `[o]` | **Organizar**: Mover los archivos seleccionados a sus carpetas |
| `[u]` | **Deshacer**: Revertir la última organización en este directorio |
| `[c]` | **Ver Formatos**: Ventana modal con todas las reglas y categorías activas |
| `[r]` | Recargar directorio |
| `[q]` o `[Esc]` | Salir de la aplicación |

---

## 🚀 Instalación y Uso

### 1. Compilar en modo Release
```bash
cargo build --release
```
El binario ejecutable estará listo en `./target/release/tidyfiles`.

### 2. Probar en una carpeta de prueba
Puedes crear una carpeta de prueba con varios archivos para ver la TUI en acción:
```bash
# Crear carpeta de prueba con archivos simulados
mkdir -p ~/prueba_descargas && touch ~/prueba_descargas/{foto.jpg,meme.png,factura.pdf,tesis.docx,video.mp4,musica.mp3,backup.zip,script.py,desconocido.custom}

# Abrir en modo TUI visual interactiva
cargo run -- ~/prueba_descargas
```

### 3. Modo CLI Directo (Sin interfaz gráfica)
Si deseas usarlo en scripts o automatizaciones:
```bash
# Simular sin mover nada (Dry Run)
cargo run -- ~/prueba_descargas --dry-run

# Ejecutar organización directa por CLI
cargo run -- ~/prueba_descargas --cli

# Deshacer directamente desde consola
cargo run -- ~/prueba_descargas --undo
```

---

## ⚙️ Personalizar Formatos y Categorías

TidyFiles utiliza un **único archivo de configuración central** en `~/.config/tidyfiles/config.toml` que aplica automáticamente a cualquier carpeta de tu equipo.

### Ver la ruta de tu configuración activa:
```bash
tidyfiles --config-path
```

### Crear o inicializar tu configuración:
```bash
tidyfiles --init-config
```

Esto generará el archivo `~/.config/tidyfiles/config.toml` con esta estructura:

```toml
[categories.Imagenes]
icon = "🖼️"
extensions = ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico", "raw"]

[categories.Documentos]
icon = "📄"
extensions = ["pdf", "docx", "doc", "txt", "xlsx", "pptx", "csv", "md"]

# ¡Puedes agregar tus propias categorías fácilmente!
[categories.Modelos3D]
icon = "🧊"
extensions = ["blend", "fbx", "obj", "stl"]

[categories.Torrents]
icon = "🧲"
extensions = ["torrent"]
```

> **Nota para proyectos aislados**: Si en alguna carpeta específica necesitas reglas especiales que no afecten al resto del sistema, puedes crear un archivo local `tidyfiles.toml` dentro de esa carpeta. TidyFiles le dará prioridad a ese directorio puntual.

---

## 🧪 Pruebas Automatizadas

El proyecto incluye tests unitarios e integrados completos:
```bash
cargo test
```
