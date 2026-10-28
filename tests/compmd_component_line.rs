use markdown::{
    mdast::{CompmdRaw, CompmdRawKind, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn compmd_component_line() -> Result<(), message::Message> {
    let compmd = Options {
        parse: ParseOptions {
            constructs: Constructs {
                compmd_component_line: true,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("::: card\na\n::: /card"),
        "<p>::: card\na\n::: /card</p>",
        "should not support component lines by default"
    );

    assert_eq!(
        to_html_with_options("::: card\na\n::: /card", &compmd)?,
        "<p>::: card</p>\n<p>a</p>\n<p>::: /card</p>",
        "should support component lines"
    );

    assert_eq!(
        to_html_with_options(":: chart[Sales] {type=\"bar\"}", &compmd)?,
        "<p>:: chart[Sales] {type=&quot;bar&quot;}</p>",
        "should support a leaf, with its text as written"
    );

    assert_eq!(
        to_html_with_options("::", &compmd)?,
        "<p>::</p>",
        "should support a line of just `::`"
    );

    assert_eq!(
        to_html_with_options("::x", &compmd)?,
        "<p>::x</p>",
        "should support a malformed line, which a CompMD processor reports"
    );

    assert_eq!(
        to_html_with_options(": a", &compmd)?,
        "<p>: a</p>",
        "should not support one colon"
    );

    assert_eq!(
        to_html_with_options("a\n::: card\nb", &compmd)?,
        "<p>a</p>\n<p>::: card</p>\n<p>b</p>",
        "should interrupt a paragraph"
    );

    assert_eq!(
        to_html_with_options("   ::: card", &compmd)?,
        "<p>::: card</p>",
        "should support three spaces of indent"
    );

    assert_eq!(
        to_html_with_options("    ::: card", &compmd)?,
        "<pre><code>::: card\n</code></pre>",
        "should not support four spaces of indent (code)"
    );

    assert_eq!(
        to_html_with_options("- a\n\n  ::: card", &compmd)?,
        "<ul>\n<li>\n<p>a</p>\n<p>::: card</p>\n</li>\n</ul>",
        "should support a component line in a list item"
    );

    assert_eq!(
        to_html_with_options("> ::: card\n> a", &compmd)?,
        "<blockquote>\n<p>::: card</p>\n<p>a</p>\n</blockquote>",
        "should support a component line in a block quote"
    );

    assert_eq!(
        to_html_with_options("> a\n::: card", &compmd)?,
        "<blockquote>\n<p>a</p>\n</blockquote>\n<p>::: card</p>",
        "should not be a lazy line"
    );

    assert_eq!(
        to_html_with_options("```\n::: card\n```", &compmd)?,
        "<pre><code>::: card\n</code></pre>",
        "should not support a component line in code"
    );

    assert_eq!(
        to_html_with_options("<div>\n::: card\n</div>", &compmd)?,
        "&lt;div&gt;\n::: card\n&lt;/div&gt;",
        "should not support a component line in HTML (flow)"
    );

    assert_eq!(
        to_html_with_options(
            "a\n::: card",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs::compmd(),
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>a</p>\n<p>::: card</p>",
        "should support component lines w/ `Constructs::compmd()`"
    );

    assert_eq!(
        to_mdast("a\n  ::: card {title=\"Hi\"}", &compmd.parse)?,
        Node::Root(Root {
            children: vec![
                Node::Paragraph(Paragraph {
                    children: vec![Node::Text(Text {
                        value: "a".into(),
                        position: Some(Position::new(1, 1, 0, 1, 2, 1))
                    })],
                    position: Some(Position::new(1, 1, 0, 1, 2, 1))
                }),
                Node::CompmdRaw(CompmdRaw {
                    kind: CompmdRawKind::ComponentLine,
                    value: "::: card {title=\"Hi\"}".into(),
                    position: Some(Position::new(2, 3, 4, 2, 24, 25))
                })
            ],
            position: Some(Position::new(1, 1, 0, 2, 24, 25))
        }),
        "should support component lines as `CompmdRaw`s in mdast"
    );

    Ok(())
}
