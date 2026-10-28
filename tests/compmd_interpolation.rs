use markdown::{
    mdast::{CompmdRawKind, Node},
    message, to_html_with_options, to_mdast, Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

/// The CompMD tokens in `value`, with what they hold.
fn raws(
    value: &str,
    options: &ParseOptions,
) -> Result<Vec<(CompmdRawKind, String)>, message::Message> {
    fn visit(node: &Node, found: &mut Vec<(CompmdRawKind, String)>) {
        if let Node::CompmdRaw(raw) = node {
            found.push((raw.kind, raw.value.clone()));
        }
        for child in node.children().into_iter().flatten() {
            visit(child, found);
        }
    }
    let mut found = vec![];
    visit(&to_mdast(value, options)?, &mut found);
    Ok(found)
}

#[test]
fn compmd_interpolation() -> Result<(), message::Message> {
    let compmd = Options {
        parse: ParseOptions {
            constructs: Constructs {
                compmd_interpolation: true,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let parse = &compmd.parse;
    let interpolation = |value: &str| (CompmdRawKind::Interpolation, value.to_string());
    let literal = (CompmdRawKind::InterpolationLiteral, "{{".to_string());

    assert_eq!(
        raws("a {{b}} c", &ParseOptions::default())?,
        vec![],
        "should not support interpolation by default"
    );

    assert_eq!(
        raws("a {{ b.c }} d", parse)?,
        vec![interpolation("{{ b.c }}")],
        "should support interpolation"
    );

    assert_eq!(
        to_html_with_options("a {{ b.c }} d", &compmd)?,
        "<p>a {{ b.c }} d</p>",
        "should write interpolation as its text in HTML"
    );

    assert_eq!(
        raws("{{a}}{{_b.c_1}}", parse)?,
        vec![interpolation("{{a}}"), interpolation("{{_b.c_1}}")],
        "should support adjacent interpolations, and `_` in names"
    );

    assert_eq!(
        raws("{{ 1 + 2 }}", parse)?,
        vec![literal.clone()],
        "should support `{{` that starts no interpolation (expression)"
    );

    assert_eq!(
        raws("{{a b}}", parse)?,
        vec![literal.clone()],
        "should not support white space in a path"
    );

    assert_eq!(
        raws("{{a.}} {{.a}} {{1a}}", parse)?,
        vec![literal.clone(), literal.clone(), literal.clone()],
        "should not support an empty name, or a name starting with a digit"
    );

    assert_eq!(
        raws("{{a}", parse)?,
        vec![literal.clone()],
        "should not support one closing brace"
    );

    assert_eq!(
        raws("{{a\n}}", parse)?,
        vec![literal],
        "should not support a line ending"
    );

    assert_eq!(
        raws("\\{{a}} {a}", parse)?,
        vec![],
        "should not support an escaped brace, or one brace"
    );

    assert_eq!(
        to_html_with_options("*{{a*}}", &compmd)?,
        "<p><em>{{a</em>}}</p>",
        "should not form an interpolation with attention markers in it"
    );

    assert_eq!(
        to_html_with_options("*{{a}}*", &compmd)?,
        "<p><em>{{a}}</em></p>",
        "should support interpolation in attention"
    );

    assert_eq!(
        raws("`{{a}}`", parse)?,
        vec![],
        "should not support interpolation in code (text)"
    );

    assert_eq!(
        raws("[{{a}}](b)", parse)?,
        vec![interpolation("{{a}}")],
        "should support interpolation in link text"
    );

    Ok(())
}
