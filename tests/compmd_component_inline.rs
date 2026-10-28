use markdown::{
    mdast::{CompmdRaw, CompmdRawKind, Node, Paragraph, Root, Text},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn compmd_component_inline() -> Result<(), message::Message> {
    let compmd = Options {
        parse: ParseOptions {
            constructs: Constructs {
                compmd_component_inline: true,
                ..Constructs::gfm()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html(":a[*b*]"),
        "<p>:a[<em>b</em>]</p>",
        "should not support inline components by default"
    );

    assert_eq!(
        to_html_with_options(":a[*b*]", &compmd)?,
        "<p>:a[*b*]</p>",
        "should support an inline component, its fallback as written"
    );

    assert_eq!(
        to_html_with_options("x :a-b_c[*d*]{e=\"*f*\" .g} h", &compmd)?,
        "<p>x :a-b_c[*d*]{e=&quot;*f*&quot; .g} h</p>",
        "should support params"
    );

    assert_eq!(
        to_html_with_options(":a[*b*] {c} *d*", &compmd)?,
        "<p>:a[*b*] {c} <em>d</em></p>",
        "should not support params after white space"
    );

    assert_eq!(
        to_html_with_options(":a[b]{{*c*}}", &compmd)?,
        "<p>:a[b]{{<em>c</em>}}</p>",
        "should not support params at `{{`, which starts an interpolation"
    );

    assert_eq!(
        to_html_with_options(":a[b]{c=\n*d*}", &compmd)?,
        "<p>:a[b]{c=\n<em>d</em>}</p>",
        "should end at the `]` when params don’t close on the line"
    );

    assert_eq!(
        to_html_with_options(":a[*b*\n*c*]", &compmd)?,
        "<p>:a[*b*\n*c*]</p>",
        "should support a fallback over several lines"
    );

    assert_eq!(
        to_html_with_options(":a[*b*", &compmd)?,
        "<p>:a[<em>b</em></p>",
        "should not support an unclosed fallback"
    );

    assert_eq!(
        to_html_with_options(":a *b*", &compmd)?,
        "<p>:a <em>b</em></p>",
        "should not support a component without a fallback"
    );

    assert_eq!(
        to_html_with_options(":1[*b*]", &compmd)?,
        "<p>:1[<em>b</em>]</p>",
        "should not support a name starting with a digit"
    );

    assert_eq!(
        to_html_with_options("x:a[*b*] 10:30[*c*]", &compmd)?,
        "<p>x:a[<em>b</em>] 10:30[<em>c</em>]</p>",
        "should not support a `:` after a letter or digit"
    );

    assert_eq!(
        to_html_with_options("a ::b[*c*]", &compmd)?,
        "<p>a ::b[<em>c</em>]</p>",
        "should not support a `:` after a `:`"
    );

    assert_eq!(
        to_html_with_options("\\:a[*b*]", &compmd)?,
        "<p>:a[<em>b</em>]</p>",
        "should not support an escaped `:`"
    );

    assert_eq!(
        to_html_with_options("(:a[*b*])", &compmd)?,
        "<p>(:a[*b*])</p>",
        "should support a `:` after punctuation"
    );

    assert_eq!(
        to_html_with_options(":a[[*b*] c]", &compmd)?,
        "<p>:a[[*b*] c]</p>",
        "should support balanced brackets in a fallback"
    );

    assert_eq!(
        to_html_with_options(":a[\\] *b*]", &compmd)?,
        "<p>:a[\\] *b*]</p>",
        "should support escaped brackets in a fallback"
    );

    assert_eq!(
        to_html_with_options(":a[b `c]` *d*]", &compmd)?,
        "<p>:a[b `c]` *d*]</p>",
        "should not end a fallback at a `]` in code (text)"
    );

    assert_eq!(
        to_html_with_options(":a[`b] *c*", &compmd)?,
        "<p>:a[`b] <em>c</em></p>",
        "should count brackets after a run of backticks that no run closes"
    );

    assert_eq!(
        to_html_with_options(":a[``b`]`` *c*]", &compmd)?,
        "<p>:a[``b`]`` *c*]</p>",
        "should only close code (text) with a run of the same size"
    );

    assert_eq!(
        to_html_with_options(":a[b]{c=(f(\")\") + '(')} *d*", &compmd)?,
        "<p>:a[b]{c=(f(&quot;)&quot;) + '(')} <em>d</em></p>",
        "should count parentheses outside strings in an expression"
    );

    assert_eq!(
        to_html_with_options(":a[b]{c=(\"\"\"x\")y\"\"\")} *d*", &compmd)?,
        "<p>:a[b]{c=(&quot;&quot;&quot;x&quot;)y&quot;&quot;&quot;)} <em>d</em></p>",
        "should support triple quoted strings in an expression"
    );

    assert_eq!(
        to_html_with_options(":a[b]{c=(r\"\\\")} *d*", &compmd)?,
        "<p>:a[b]{c=(r&quot;\\&quot;)} <em>d</em></p>",
        "should support raw strings in an expression"
    );

    assert_eq!(
        to_html_with_options(":a[b]{c=(\"\\\")\")} *d*", &compmd)?,
        "<p>:a[b]{c=(&quot;\\&quot;)&quot;)} <em>d</em></p>",
        "should support escapes in strings in an expression"
    );

    assert_eq!(
        to_html_with_options(":a[b]{c=[\"}\", {\"d\": 1}]} *e*", &compmd)?,
        "<p>:a[b]{c=[&quot;}&quot;, {&quot;d&quot;: 1}]} <em>e</em></p>",
        "should support JSON values"
    );

    assert_eq!(
        to_html_with_options("# :a[*b*]", &compmd)?,
        "<h1>:a[*b*]</h1>",
        "should support an inline component in a heading"
    );

    assert_eq!(
        to_html_with_options("| :a[*b*]{c=(d \\|\\| e)} |\n| - |", &compmd)?,
        "<table>\n<thead>\n<tr>\n<th>:a[*b*]{c=(d \\|\\| e)}</th>\n</tr>\n</thead>\n</table>",
        "should support an inline component in a table cell, `|` escaped"
    );

    assert_eq!(
        to_html_with_options("`:a[*b*]`", &compmd)?,
        "<p><code>:a[*b*]</code></p>",
        "should not support an inline component in code (text)"
    );

    assert_eq!(
        to_mdast("x :a[b]{c=1} y", &compmd.parse)?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![
                    Node::Text(Text {
                        value: "x ".into(),
                        position: Some(Position::new(1, 1, 0, 1, 3, 2))
                    }),
                    Node::CompmdRaw(CompmdRaw {
                        kind: CompmdRawKind::ComponentInline,
                        value: ":a[b]{c=1}".into(),
                        position: Some(Position::new(1, 3, 2, 1, 13, 12))
                    }),
                    Node::Text(Text {
                        value: " y".into(),
                        position: Some(Position::new(1, 13, 12, 1, 15, 14))
                    })
                ],
                position: Some(Position::new(1, 1, 0, 1, 15, 14))
            })],
            position: Some(Position::new(1, 1, 0, 1, 15, 14))
        }),
        "should support inline components as `CompmdRaw`s in mdast"
    );

    Ok(())
}
