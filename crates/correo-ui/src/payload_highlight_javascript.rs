pub(super) fn is_js_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || matches!(ch, '_' | '$')
}

pub(super) fn is_js_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$')
}

pub(super) fn is_js_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '{' | '}'
            | '['
            | ']'
            | '('
            | ')'
            | ';'
            | ','
            | '.'
            | ':'
            | '?'
            | '!'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '='
            | '<'
            | '>'
            | '&'
            | '|'
    )
}

pub(super) fn is_js_keyword(word: &str) -> bool {
    matches!(
        word,
        "async"
            | "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "from"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "let"
            | "new"
            | "null"
            | "of"
            | "return"
            | "static"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "undefined"
            | "var"
            | "void"
            | "while"
            | "yield"
    )
}

pub(super) fn is_class_like(word: &str) -> bool {
    word.chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

pub(super) fn previous_word_is_new(text: &str, index: usize) -> bool {
    let Some(prefix) = text.get(..index) else {
        return false;
    };
    let trimmed = prefix.trim_end();
    let Some(end) = trimmed
        .char_indices()
        .last()
        .map(|(index, ch)| index + ch.len_utf8())
    else {
        return false;
    };
    let start = trimmed[..end]
        .char_indices()
        .rev()
        .find(|(_, ch)| !is_js_ident_continue(*ch))
        .map_or(0, |(index, ch)| index + ch.len_utf8());
    &trimmed[start..end] == "new"
}
