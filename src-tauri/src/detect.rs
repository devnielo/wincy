//! Content-kind and secret detection for captured text (see docs/ARCHITECTURE.md).

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

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

const MAX_SECRET_SCAN_BYTES: usize = 8 * 1024;

/// Name of the secret pattern the text matches, if any. High precision only: missing a
/// secret is acceptable, silently dropping an ordinary clip is not.
pub fn looks_secret(text: &str) -> Option<&'static str> {
    let t = text.trim();
    if t.len() > MAX_SECRET_SCAN_BYTES {
        return None;
    }
    if has_private_key_block(t) {
        return Some("private key");
    }
    if is_card_number(t) {
        return Some("payment card");
    }
    // Tokens are scanned anywhere, so `KEY=value` lines and `Bearer <jwt>` headers count too.
    t.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')))
        .map(|w| w.trim_matches('.'))
        .find_map(token_kind)
}

fn has_private_key_block(t: &str) -> bool {
    t.match_indices("-----BEGIN ").any(|(i, m)| {
        t[i + m.len()..]
            .split_once("-----")
            .is_some_and(|(label, _)| {
                label.ends_with("PRIVATE KEY") || label.ends_with("PRIVATE KEY BLOCK")
            })
    })
}

/// 13–19 digits with optional spaces/dashes that pass Luhn. Only major-network first digits
/// (2–6), so millisecond timestamps and other long ids starting with 1 never match.
fn is_card_number(t: &str) -> bool {
    if !t
        .chars()
        .all(|c| c.is_ascii_digit() || c == ' ' || c == '-')
    {
        return false;
    }
    let digits: Vec<u32> = t.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&digits.len()) || !(2..=6).contains(&digits[0]) {
        return false;
    }
    let sum: u32 = digits
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &d)| match (i % 2, d * 2) {
            (0, _) => d,
            (_, x) if x > 9 => x - 9,
            (_, x) => x,
        })
        .sum();
    sum.is_multiple_of(10)
}

/// `prefix` + a rest of exactly / at least `len` characters, all accepted by `ok`.
fn prefixed(w: &str, prefixes: &[&str], ok: fn(char) -> bool, len: fn(usize) -> bool) -> bool {
    prefixes.iter().any(|p| {
        w.strip_prefix(p)
            .is_some_and(|r| len(r.len()) && r.chars().all(ok))
    })
}

fn token_kind(w: &str) -> Option<&'static str> {
    let alnum = |c: char| c.is_ascii_alphanumeric();
    let upper_or_digit = |c: char| c.is_ascii_uppercase() || c.is_ascii_digit();
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let dashed = |c: char| c.is_ascii_alphanumeric() || c == '-';
    let key = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';

    if prefixed(w, &["AKIA", "ASIA"], upper_or_digit, |n| n == 16) {
        Some("aws access key")
    } else if prefixed(w, &["ghp_", "gho_", "ghu_", "ghs_", "ghr_"], alnum, |n| {
        n == 36
    }) || prefixed(w, &["github_pat_"], word, |n| n >= 40)
    {
        Some("github token")
    } else if prefixed(
        w,
        &["xoxa-", "xoxb-", "xoxp-", "xoxr-", "xoxs-"],
        dashed,
        |n| n >= 20,
    ) {
        Some("slack token")
    } else if prefixed(w, &["sk_live_", "rk_live_"], alnum, |n| n >= 20) {
        Some("stripe key")
    } else if prefixed(w, &["AIza"], key, |n| n == 35) {
        Some("google api key")
    } else if prefixed(w, &["sk-proj-", "sk-ant-"], key, |n| n >= 20) {
        Some("ai api key")
    } else if is_jwt(w) {
        Some("jwt")
    } else {
        None
    }
}

/// Three base64url segments whose header is a JSON object with an `alg` field.
fn is_jwt(w: &str) -> bool {
    let parts: Vec<&str> = w.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| !p.is_empty())
        && URL_SAFE_NO_PAD.decode(parts[0]).is_ok_and(|h| {
            serde_json::from_slice::<serde_json::Map<String, serde_json::Value>>(&h)
                .is_ok_and(|m| m.contains_key("alg"))
        })
}

#[cfg(test)]
mod tests {
    use super::{detect, looks_secret};

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

    #[test]
    fn secrets_are_detected() {
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
                   eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.\
                   SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let bearer = format!("Authorization: Bearer {jwt}");
        // Token-shaped fixtures are split with concat! so secret scanners (GitHub push protection) do not flag these fakes.
        let cases = [
            (
                "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA\n-----END RSA PRIVATE KEY-----",
                "private key",
            ),
            (
                "-----BEGIN OPENSSH PRIVATE KEY-----\nb3BlbnNzaC1rZXktdjEA\n-----END OPENSSH PRIVATE KEY-----",
                "private key",
            ),
            ("-----BEGIN PRIVATE KEY-----\nMIIE", "private key"),
            ("-----BEGIN PGP PRIVATE KEY BLOCK-----", "private key"),
            (jwt, "jwt"),
            (&bearer, "jwt"),
            ("AKIAIOSFODNN7EXAMPLE", "aws access key"),
            ("ASIAY34FZKBOKMUTVV7A", "aws access key"),
            ("aws_access_key_id = AKIAIOSFODNN7EXAMPLE", "aws access key"),
            ("ghp_16C7e42F292c6912E7710c838347Ae178B4a", "github token"),
            ("gho_16C7e42F292c6912E7710c838347Ae178B4a", "github token"),
            ("ghs_16C7e42F292c6912E7710c838347Ae178B4a", "github token"),
            (
                "github_pat_11ABCDEFG0123456789_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123",
                "github token",
            ),
            (
                concat!(
                    "xo",
                    "xb-123456789012-1234567890123-AbCdEfGhIjKlMnOpQrStUvWx"
                ),
                "slack token",
            ),
            (
                concat!("xo", "xp-123456789012-123456789012-123456789012-abcdef"),
                "slack token",
            ),
            (
                concat!("sk", "_live_4eC39HqLyjWDarjtT1zdp7dc"),
                "stripe key",
            ),
            (
                concat!("rk", "_live_4eC39HqLyjWDarjtT1zdp7dcXYZ"),
                "stripe key",
            ),
            ("AIzaSyDdI0hCZtE6vySjMm-WEfRq3CPzqKqqsHI", "google api key"),
            ("sk-proj-abcDEF123_ghiJKL456-mnoPQR789stu", "ai api key"),
            (
                "sk-ant-api03-abcdefghijklmnopqrstuvwxyz0123456789",
                "ai api key",
            ),
            (
                "OPENAI_API_KEY=sk-proj-abcDEF123_ghiJKL456-mnoPQR789stu",
                "ai api key",
            ),
            ("4111 1111 1111 1111", "payment card"),
            ("4111-1111-1111-1111", "payment card"),
            ("5500005555555559", "payment card"),
            ("378282246310005", "payment card"),
            ("  6011111111111117  ", "payment card"),
        ];
        for (text, name) in cases {
            assert_eq!(looks_secret(text), Some(name), "{text}");
        }
    }

    #[test]
    fn ordinary_text_is_not_secret() {
        let base64_png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg=="
            .repeat(20);
        let base64_text =
            "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4u"
                .repeat(10);
        let samples = [
            "Hello there, the meeting moved to 3 pm. Bring the AKIA report.",
            "The private key to success is patience.",
            "-----BEGIN CERTIFICATE-----\nMIIDdzCCAl+gAwIBAgIE\n-----END CERTIFICATE-----",
            "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkq\n-----END PUBLIC KEY-----",
            "550e8400-e29b-41d4-a716-446655440000",
            "f47ac10b-58cc-4372-a567-0e02b2c3d479",
            "da39a3ee5e6b4b0d3255bfef95601890afd80709",
            "commit 2fd4e1c67a2d28fced849ee1bb76e7391b93eb12",
            "+1 (555) 123-4567",
            "+34 612 345 678",
            "555-123-4567",
            "4111 1111 1111 1112",
            "1234567890123456",
            "1732550400000",
            "https://example.com/docs/getting-started?lang=en#install",
            "www.github.com/devnielo/wincy",
            &base64_png,
            &base64_text,
            "a.b.c",
            "eyJmb28iOiJiYXIifQ.eyJzdWIiOiIxIn0.c2ln",
            "AKIAIOSFODNN7EXAMPL",
            "ghp_short",
            "sk-proj-short",
            concat!("sk", "_test_4eC39HqLyjWDarjtT1zdp7dc"),
            "",
        ];
        for s in samples {
            assert_eq!(looks_secret(s), None, "{s}");
        }
        let huge = format!("{}AKIAIOSFODNN7EXAMPLE", "x ".repeat(5000));
        assert_eq!(looks_secret(&huge), None);
    }
}
