use markdown::{
    mdast::{CompmdRaw, CompmdRawKind, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn compmd_wiki_link() -> Result<(), message::Message> {
    let compmd = Options {
        parse: ParseOptions {
            constructs: Constructs {
                compmd_wiki_link: true,
                ..Constructs::gfm()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("[[*a*]]"),
        "<p>[[<em>a</em>]]</p>",
        "should not support wiki links by default"
    );

    assert_eq!(
        to_html_with_options("x [[*a*]] y", &compmd)?,
        "<p>x [[*a*]] y</p>",
        "should support a wiki link"
    );

    assert_eq!(
        to_html_with_options("[[a b|*c*]]", &compmd)?,
        "<p>[[a b|*c*]]</p>",
        "should support a wiki link with a label"
    );

    assert_eq!(
        to_html_with_options("[[a]]\n\n[a]: /x", &compmd)?,
        "<p>[[a]]</p>\n",
        "should be a wiki link, not a link in brackets"
    );

    assert_eq!(
        to_html("[[a]]\n\n[a]: /x"),
        "<p>[<a href=\"/x\">a</a>]</p>\n",
        "should be a link in brackets without wiki links"
    );

    assert_eq!(
        to_html_with_options("[[*a*|]] [[|*b*]] [[]]", &compmd)?,
        "<p>[[<em>a</em>|]] [[|<em>b</em>]] [[]]</p>",
        "should not support an empty target or label"
    );

    assert_eq!(
        to_html_with_options("[[a|b|*c*]]", &compmd)?,
        "<p>[[a|b|<em>c</em>]]</p>",
        "should not support `|` in the label"
    );

    assert_eq!(
        to_html_with_options("[[*a*]b]]", &compmd)?,
        "<p>[[<em>a</em>]b]]</p>",
        "should not support `]` in the target"
    );

    assert_eq!(
        to_html_with_options("[[*a*\n]]", &compmd)?,
        "<p>[[<em>a</em>\n]]</p>",
        "should not support a line ending"
    );

    assert_eq!(
        to_html_with_options("[[*a*]", &compmd)?,
        "<p>[[<em>a</em>]</p>",
        "should not support one closing bracket"
    );

    assert_eq!(
        to_html_with_options("[^a] [b](c)\n\n[^a]: d", &compmd)?,
        to_html_with_options("[^a] [b](c)\n\n[^a]: d", &Options::gfm())?,
        "should still support footnotes and links"
    );

    assert_eq!(
        to_html_with_options("| [[a\\|*b*]] |\n| - |", &compmd)?,
        "<table>\n<thead>\n<tr>\n<th>[[a\\|*b*]]</th>\n</tr>\n</thead>\n</table>",
        "should support a wiki link in a table cell, `|` escaped"
    );

    assert_eq!(
        to_mdast("x [[a|b]]", &compmd.parse)?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "x ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::CompmdRaw(CompmdRaw {
                        kind: CompmdRawKind::WikiLink,
                        value: "[[a|b]]".into(),
                        position: Some(Position::new(1, 3, 2, 1, 10, 9))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 10, 9))
            })],
            position: Some(Position::new(1, 1, 0, 1, 10, 9))
        }),
        "should support wiki links as `CompmdRaw`s in mdast"
    );

    Ok(())
}
