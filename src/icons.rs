/// Devuelve un icono seguro para terminales de solo texto.
///
/// Las configuraciones antiguas pueden traer emojis o iconos vacíos. En ese
/// caso se usa una abreviatura ASCII basada en el nombre de la categoría.
pub fn display_icon(icon: &str, category: &str) -> String {
    let trimmed = icon.trim();
    if !trimmed.is_empty() && trimmed.is_ascii() && trimmed.chars().all(|c| c.is_ascii_graphic()) {
        return trimmed.chars().take(4).collect();
    }

    let mut fallback: String = category
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(3)
        .collect();
    fallback.make_ascii_uppercase();

    if fallback.is_empty() {
        "DIR".to_string()
    } else {
        fallback
    }
}

#[cfg(test)]
mod tests {
    use super::display_icon;

    #[test]
    fn keeps_ascii_icons() {
        assert_eq!(display_icon("IMG", "Imagenes"), "IMG");
        assert_eq!(display_icon("CODE", "Codigo"), "CODE");
    }

    #[test]
    fn replaces_emoji_icons_with_ascii_fallback() {
        assert_eq!(display_icon("🎬", "Videos"), "VID");
        assert_eq!(display_icon("⚙️", "Instaladores"), "INS");
    }

    #[test]
    fn replaces_blank_icons_with_category_fallback() {
        assert_eq!(display_icon(" ", "Video Procesado"), "VID");
        assert_eq!(display_icon("", "Otros"), "OTR");
    }
}
