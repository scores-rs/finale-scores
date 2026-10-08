//! Enigma strings: the text of text blocks, expressions and file info,
//! with formatting commands inline. A command is `^name(args)`, such as
//! `^fontid(1)`, `^size(14)` or `^flat()`; `^^` is a literal caret.

/// A piece of Enigma text: literal text, or a `^name(args)` command.
enum Token<'a> {
    Text(&'a str),
    Command { name: &'a str, args: &'a str },
}

/// Splits Enigma text into literal runs and commands; `^^` is a literal
/// caret.
fn tokens(enigma: &str) -> Vec<Token<'_>> {
    let mut tokens = Vec::new();
    let mut rest = enigma;
    while let Some(caret) = rest.find('^') {
        if caret > 0 {
            tokens.push(Token::Text(&rest[..caret]));
        }
        rest = &rest[caret + 1..];
        if rest.starts_with('^') {
            tokens.push(Token::Text(&rest[..1]));
            rest = &rest[1..];
            continue;
        }
        let name_len = rest
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(rest.len());
        let name = &rest[..name_len];
        let after_name = &rest[name_len..];
        let Some(args) = after_name.strip_prefix('(') else {
            rest = after_name;
            continue;
        };
        let close = args.find(')').unwrap_or(args.len());
        tokens.push(Token::Command {
            name,
            args: &args[..close],
        });
        rest = args.get(close + 1..).unwrap_or("");
    }
    if !rest.is_empty() {
        tokens.push(Token::Text(rest));
    }
    tokens
}

/// Strips Enigma text commands (`^fontid(1)`, `^size(14)`, ...) from a
/// text block, substituting the accidental commands.
pub fn plain_text(enigma: &str) -> String {
    let mut out = String::new();
    for token in tokens(enigma) {
        match token {
            Token::Text(text) => out.push_str(text),
            Token::Command { name: "flat", .. } => out.push('♭'),
            Token::Command { name: "sharp", .. } => out.push('♯'),
            Token::Command {
                name: "natural", ..
            } => out.push('♮'),
            Token::Command { .. } => {}
        }
    }
    out.trim().to_string()
}

/// A font named in Enigma text: an id into `others/fontName`, or a family.
#[derive(Debug, Clone, PartialEq)]
pub enum FontRef {
    Id(u32),
    Name(String),
}

/// Enigma text split into runs by the font they're drawn in (`None`
/// before the first font command).
pub fn font_runs(enigma: &str) -> Vec<(Option<FontRef>, String)> {
    let mut runs: Vec<(Option<FontRef>, String)> = Vec::new();
    let mut font = None;
    for token in tokens(enigma) {
        match token {
            Token::Text(text) => match runs.last_mut() {
                Some((f, run)) if *f == font => run.push_str(text),
                _ => runs.push((font.clone(), text.to_string())),
            },
            Token::Command { name, args } => match name {
                "fontid" => font = args.trim().parse().ok().map(FontRef::Id),
                "fontMus" | "fontTxt" | "fontNum" | "font" | "Font" => {
                    let first = args.split(',').next().unwrap_or("").trim();
                    font = Some(match first.strip_prefix("Font").map(str::parse) {
                        Some(Ok(id)) => FontRef::Id(id),
                        _ => FontRef::Name(first.to_string()),
                    });
                }
                _ => {}
            },
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_commands() {
        assert_eq!(
            plain_text("^fontid(1)^size(14)^nfx(0)B^flat() Cl."),
            "B♭ Cl."
        );
        assert_eq!(plain_text("2^^3"), "2^3");
    }

    #[test]
    fn splits_by_font() {
        assert_eq!(
            font_runs("^fontTxt(Times,4096)^size(12)subito ^fontMus(Font0,0)p"),
            vec![
                (Some(FontRef::Name("Times".into())), "subito ".into()),
                (Some(FontRef::Id(0)), "p".into()),
            ]
        );
    }
}
