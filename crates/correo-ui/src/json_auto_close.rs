/// Completes or types over a bracket or quote the user just typed.
///
/// `cursor` is the character index right after `typed`. The cursor never has to move.
pub(crate) fn apply(text: &mut String, cursor: usize, typed: char) {
    let after = text
        .char_indices()
        .nth(cursor)
        .map_or(text.len(), |(index, _)| index);
    let Some(before) = text[..after].strip_suffix(typed) else {
        return;
    };
    if before.ends_with('\\') {
        return;
    }
    let inside_string = scan(before).inside_string;
    let next = text[after..].chars().next();
    let can_insert = next.is_none_or(|ch| ch.is_whitespace() || ",}]:".contains(ch));
    match typed {
        '"' if inside_string => {
            if next == Some('"') {
                text.remove(after);
            }
        }
        '"' if can_insert => text.insert(after, '"'),
        '{' if !inside_string && can_insert => text.insert(after, '}'),
        '[' if !inside_string && can_insert => text.insert(after, ']'),
        '}' | ']' if !inside_string && next == Some(typed) => {
            let counts = scan(text);
            // Only swallow the key press when the closer is already there, not when one is missing.
            if counts.closers > counts.openers {
                text.remove(after);
            }
        }
        _ => {}
    }
}

#[derive(Default)]
struct Scan {
    inside_string: bool,
    openers: usize,
    closers: usize,
}

fn scan(text: &str) -> Scan {
    let mut scan = Scan::default();
    let mut escaped = false;
    for ch in text.chars() {
        if scan.inside_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                scan.inside_string = false;
            }
            continue;
        }
        match ch {
            '"' => scan.inside_string = true,
            '{' | '[' => scan.openers += 1,
            '}' | ']' => scan.closers += 1,
            _ => {}
        }
    }
    scan
}

#[cfg(test)]
mod tests {
    use super::apply;

    /// `|` marks the cursor right after the typed character.
    fn typed(marked: &str) -> String {
        let cursor = marked.chars().position(|ch| ch == '|').expect("cursor");
        let mut text = marked.replace('|', "");
        let typed = text.chars().nth(cursor - 1).expect("typed character");
        apply(&mut text, cursor, typed);
        text
    }

    #[test]
    fn closes_brackets_and_quotes() {
        assert_eq!(typed("{|"), "{}");
        assert_eq!(typed("{\"a\": [|}"), "{\"a\": []}");
        assert_eq!(typed("{\"|}"), "{\"\"}");
    }

    #[test]
    fn types_over_a_closer_that_is_already_there() {
        assert_eq!(typed("{\"a\"|\"}"), "{\"a\"}");
        assert_eq!(typed("{\"a\": 1}|}"), "{\"a\": 1}");
    }

    #[test]
    fn keeps_a_closer_that_was_missing() {
        assert_eq!(typed("{\"a\": {\"b\": 1}|}"), "{\"a\": {\"b\": 1}}");
    }

    #[test]
    fn leaves_text_inside_strings_and_before_other_text_alone() {
        assert_eq!(typed("{\"a{|\"}"), "{\"a{\"}");
        assert_eq!(typed("{\"a\\\"|\"}"), "{\"a\\\"\"}");
        assert_eq!(typed("[|1]"), "[1]");
        assert_eq!(typed("{\"ä\": \"|}"), "{\"ä\": \"\"}");
    }
}
