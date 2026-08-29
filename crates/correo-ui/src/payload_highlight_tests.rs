use super::*;

#[test]
fn highlight_key_changes_with_payload_or_plugins() {
    let plugins = vec!["json".to_owned()];
    assert_ne!(
        HighlightKey::new("{\"id\":1}", &plugins),
        HighlightKey::new("{\"id\":2}", &plugins)
    );
    assert_ne!(
        HighlightKey::new("{\"id\":1}", &plugins),
        HighlightKey::new("{\"id\":1}", &["xml".to_owned()])
    );
}

#[test]
fn malformed_plugin_span_does_not_split_utf8_codepoint() {
    let palette = Palette {
        plain: Color32::WHITE,
        key: Color32::WHITE,
        string: Color32::WHITE,
        number: Color32::WHITE,
        keyword: Color32::WHITE,
        punct: Color32::WHITE,
        tag: Color32::WHITE,
        attr: Color32::WHITE,
        comment: Color32::WHITE,
        method: Color32::WHITE,
        class: Color32::WHITE,
    };
    let spans = [PayloadSyntaxSpan {
        start: 1,
        end: 2,
        kind: PayloadSyntaxKind::String,
    }];

    let job = highlight_spans("éx", &spans, FontId::default(), palette);

    assert_eq!(job.text, "éx");
}
