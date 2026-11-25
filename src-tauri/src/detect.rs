//! Content-kind detection for captured text (see "Kind detection" in docs/ARCHITECTURE.md).

pub fn detect(text: &str) -> &'static str {
    let t = text.trim();
    if is_color(t) {
        "color"
    } else if is_url(t) {
        "url"
    } else if is_code(text) {
        "code"
    } else {
        "text"
    }
}

fn is_color(t: &str) -> bool {
    if let Some(hex) = t.strip_prefix('#') {
        return matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    let lower = t.to_ascii_lowercase();
    let Some((hsl, inner)) = ["rgba(", "rgb(", "hsla(", "hsl("].iter().find_map(|p| {
        Some((
            p.starts_with('h'),
            lower.strip_prefix(p)?.strip_suffix(')')?,
        ))
    }) else {
        return false;
    };
    // Same grammar as parseColor in src/lib/color.ts, so every detected color gets a swatch.
    // Legacy comma syntax or modern `rgb(0 0 0 / 50%)` syntax.
    let mut slash = inner.split('/');
    let main = slash.next().unwrap_or_default();
    let alpha = slash.next();
    if slash.next().is_some() {
        return false;
    }
    let mut parts: Vec<&str> = if main.contains(',') {
        main.split(',').map(str::trim).collect()
    } else {
        main.split_whitespace().collect()
    };
    if let Some(a) = alpha {
        if parts.len() != 3 {
            return false;
        }
        parts.push(a.trim());
    }
    (3..=4).contains(&parts.len())
        && parts.iter().enumerate().all(|(i, p)| match (hsl, i) {
            (true, 0) => is_number(
                ["deg", "turn", "rad"]
                    .iter()
                    .find_map(|u| p.strip_suffix(u))
                    .unwrap_or(p),
            ),
            (true, 1 | 2) => p.strip_suffix('%').is_some_and(is_number),
            _ => is_number(p.strip_suffix('%').unwrap_or(p)),
        })
}

/// `[+-]?(\d+\.?\d*|\.\d+)`: plain decimal, no exponent.
fn is_number(s: &str) -> bool {
    let s = s.strip_prefix(['+', '-']).unwrap_or(s);
    let digits = |d: &str| d.bytes().all(|b| b.is_ascii_digit());
    match s.split_once('.') {
        Some((int, frac)) => digits(int) && digits(frac) && !(int.is_empty() && frac.is_empty()),
        None => !s.is_empty() && digits(s),
    }
}

fn is_url(t: &str) -> bool {
    if t.chars().any(char::is_whitespace) {
        return false;
    }
    let lower = t.to_ascii_lowercase();
    ["http://", "https://", "ftp://", "mailto:", "www."]
        .iter()
        .any(|p| lower.len() > p.len() && lower.starts_with(p))
}

// Only words that rarely start a line of wrapped prose ("from", "use", "type"… were left out on purpose).
const KEYWORDS: &[&str] = &[
    "function ",
    "const ",
    "let ",
    "var ",
    "fn ",
    "pub ",
    "def ",
    "class ",
    "import ",
    "export ",
    "return ",
    "return;",
    "package ",
    "public ",
    "private ",
    "protected ",
    "struct ",
    "enum ",
    "impl ",
    "interface ",
    "#include",
    "#define",
    "if (",
    "if(",
    "for (",
    "for(",
    "while (",
    "while(",
    "} else",
    "try {",
    "switch (",
    "<div",
    "<span",
    "<html",
    "<?php",
    "SELECT ",
    "INSERT ",
    "UPDATE ",
    "DELETE ",
    "CREATE ",
    "ALTER ",
    "DROP ",
    "FROM ",
    "WHERE ",
    "JOIN ",
    "ORDER BY",
    "GROUP BY",
    "//",
    "/*",
    "*/",
];

/// Multi-line text where at least half of the non-empty lines carry a code signal.
fn is_code(text: &str) -> bool {
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.len() < 2 {
        return false;
    }
    let signals = lines.iter().filter(|l| line_has_code_signal(l)).count();
    signals * 2 >= lines.len()
}

fn line_has_code_signal(line: &str) -> bool {
    let t = line.trim();
    let indented = line.starts_with('\t') || line.starts_with("  ");
    t.ends_with(';')
        || t.ends_with('{')
        || t.ends_with('}')
        || t.ends_with("};")
        || t.ends_with("),")
        || (t.starts_with('<') && t.ends_with('>'))
        || t.contains("=>")
        || t.contains("::")
        || KEYWORDS.iter().any(|k| t.starts_with(k))
        // Indented lines ending like a statement (Python bodies, YAML-ish blocks).
        || (indented && (t.ends_with(':') || t.ends_with(')') || t.contains(" = ")))
}

#[cfg(test)]
mod tests {
    use super::detect;

    #[test]
    fn hex_colors() {
        for c in ["#fff", "#FFFA", "#1a2b3c", "#1a2b3c80", "  #abc  "] {
            assert_eq!(detect(c), "color", "{c}");
        }
        for c in [
            "#ff",
            "#fffff",
            "#1234567",
            "#123456789",
            "#ggg",
            "#",
            "fff",
        ] {
            assert_ne!(detect(c), "color", "{c}");
        }
    }

    #[test]
    fn functional_colors() {
        for c in [
            "rgb(255, 0, 0)",
            "rgba(255,0,0,0.5)",
            "RGB(10 20 30)",
            "rgb(10 20 30 / 50%)",
            "hsl(120, 50%, 50%)",
            "hsla(120deg, 50%, 50%, 0.3)",
            "hsl(.5turn 50% 50% / .5)",
            "rgb(+1.5, 2., 3%)",
        ] {
            assert_eq!(detect(c), "color", "{c}");
        }
        for c in [
            "rgb(1,2)",
            "rgb(a,b,c)",
            "rgb(1,2,3",
            "rgb(1,2,3,4,5)",
            "hsl()",
            "hsl(120, 50, 50)",
            "rgb(1e2, 0, 0)",
            "rgb(1,,2,3)",
            "rgb(1 2 / 3)",
            "rgb(1 2 3 / 4 / 5)",
            "rgb(., 0, 0)",
            "rgb(1deg, 0, 0)",
        ] {
            assert_ne!(detect(c), "color", "{c}");
        }
    }

    #[test]
    fn urls() {
        for u in [
            "https://example.com/a?b=c",
            "http://localhost:3000",
            "ftp://files.example.com",
            "mailto:me@example.com",
            "www.example.com",
            "  HTTPS://EXAMPLE.COM  ",
        ] {
            assert_eq!(detect(u), "url", "{u}");
        }
        for u in [
            "https://",
            "see https://example.com",
            "example.com",
            "https://a.com\nhttps://b.com",
        ] {
            assert_ne!(detect(u), "url", "{u}");
        }
    }

    #[test]
    fn code() {
        let samples = [
            "function add(a, b) {\n  return a + b;\n}",
            "const x = 1;\nconst y = 2;",
            "def add(a, b):\n    return a + b",
            "fn main() {\n    println!(\"hi\");\n}",
            "SELECT id, name\nFROM users\nWHERE id = 1",
            "<div class=\"a\">\n  <span>hi</span>\n</div>",
            "{\n  \"a\": 1,\n  \"b\": 2\n}",
            "import os\nimport sys",
        ];
        for s in samples {
            assert_eq!(detect(s), "code", "{s}");
        }
    }

    #[test]
    fn prose_is_text() {
        let samples = [
            "Hello there; how are you?\nI hope the trip went well.",
            "I went to the store; it was closed.\nThen I went home and slept.",
            "Shopping list:\n- milk\n- eggs\n- bread",
            "Return the book tomorrow.\nClass starts at nine.",
            "Dear Ana,\n\nThanks for the update.\n\nBest regards,\nDaniel",
            "const x = 1;",
            "just one line of text",
        ];
        for s in samples {
            assert_eq!(detect(s), "text", "{s}");
        }
    }

    #[test]
    fn multiline_plain_text() {
        assert_eq!(detect("line one\nline two\nline three"), "text");
        assert_eq!(detect("   \n\t\n"), "text");
    }
}
