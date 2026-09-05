use anyhow::{bail, Result};

/// A single token of a compiled SQL `LIKE` pattern.
enum Tok {
    /// A literal character (already lower-cased), matches itself exactly.
    Lit(char),
    /// `_`: matches exactly one character.
    Any,
    /// `%`: matches any sequence of characters (including none).
    Star,
}

/// Compile a SQL `LIKE` pattern (`%` = any sequence, `_` = any single
/// character, `\` escapes the next character) into matchable tokens.
/// Matching is case-insensitive.
fn compile(pattern: &str) -> Result<Vec<Tok>> {
    let mut toks = Vec::new();
    let mut chars = pattern.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some(escaped) => toks.push(Tok::Lit(escaped.to_ascii_lowercase())),
                None => {
                    bail!("invalid LIKE pattern: pattern ends with a trailing '\\' escape character");
                }
            },
            '%' => toks.push(Tok::Star),
            '_' => toks.push(Tok::Any),
            other => toks.push(Tok::Lit(other.to_ascii_lowercase())),
        }
    }
    Ok(toks)
}

/// Match `text` against a SQL `LIKE` `pattern`, case-insensitively.
pub(crate) fn like_match(text: &str, pattern: &str) -> Result<bool> {
    let toks = compile(pattern)?;
    let text: Vec<char> = text.to_ascii_lowercase().chars().collect();
    let n = text.len();
    let m = toks.len();

    // Standard wildcard-matching dynamic program: dp[i][j] is true if the
    // first i characters of `text` match the first j tokens of the pattern.
    let mut dp = vec![vec![false; m + 1]; n + 1];
    dp[0][0] = true;
    for (j, tok) in toks.iter().enumerate() {
        if let Tok::Star = tok {
            dp[0][j + 1] = dp[0][j];
        }
    }
    for i in 1..=n {
        for j in 1..=m {
            dp[i][j] = match toks[j - 1] {
                Tok::Star => dp[i - 1][j] || dp[i][j - 1],
                Tok::Any => dp[i - 1][j - 1],
                Tok::Lit(c) => dp[i - 1][j - 1] && text[i - 1] == c,
            };
        }
    }
    Ok(dp[n][m])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        assert!(like_match("Foo::Bar", "Foo::Bar").unwrap());
        assert!(like_match("Foo::Bar", "foo::bar").unwrap());
        assert!(!like_match("Foo::Bar", "Foo::Baz").unwrap());
    }

    #[test]
    fn percent_wildcard() {
        assert!(like_match("Foo::Bar", "Foo::%").unwrap());
        assert!(like_match("Foo::Bar", "%::Bar").unwrap());
        assert!(like_match("Foo::Bar", "%").unwrap());
        assert!(!like_match("Foo::Bar", "Baz::%").unwrap());
    }

    #[test]
    fn underscore_wildcard() {
        assert!(like_match("Foo", "F_o").unwrap());
        assert!(!like_match("Foo", "F_").unwrap());
    }

    #[test]
    fn escaped_literal() {
        assert!(like_match("100%", "100\\%").unwrap());
        assert!(!like_match("100x", "100\\%").unwrap());
    }

    #[test]
    fn trailing_escape_is_error() {
        assert!(like_match("foo", "foo\\").is_err());
    }
}
