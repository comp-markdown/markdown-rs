use markdown::{
    mdast::{InlineMath, Node, Paragraph, Root},
    message, to_html, to_html_with_options, to_mdast,
    unist::Position,
    Constructs, Options, ParseOptions,
};
use pretty_assertions::assert_eq;

#[test]
fn compmd_math_text() -> Result<(), message::Message> {
    let compmd = Options {
        parse: ParseOptions {
            constructs: Constructs {
                compmd_math_text: true,
                ..Constructs::gfm()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    assert_eq!(
        to_html("$a$"),
        "<p>$a$</p>",
        "should not support math (text) by Pandoc’s rule by default"
    );

    assert_eq!(
        to_html_with_options("a $x^2$ b", &compmd)?,
        "<p>a <code class=\"language-math math-inline\">x^2</code> b</p>",
        "should support math (text)"
    );

    assert_eq!(
        to_html_with_options("It costs $5 and $6. Math $x^2$", &compmd)?,
        "<p>It costs $5 and $6. Math <code class=\"language-math math-inline\">x^2</code></p>",
        "should end an attempt at a `$` after white space"
    );

    assert_eq!(
        to_html_with_options("costs $5 and $10", &compmd)?,
        "<p>costs $5 and $10</p>",
        "should not support dollar amounts"
    );

    assert_eq!(
        to_html_with_options("$ a$", &compmd)?,
        "<p>$ a$</p>",
        "should not support white space after the opening `$`"
    );

    assert_eq!(
        to_html_with_options("$a $", &compmd)?,
        "<p>$a $</p>",
        "should not support white space before the closing `$`"
    );

    assert_eq!(
        to_html_with_options("$a$1$", &compmd)?,
        "<p><code class=\"language-math math-inline\">a$1</code></p>",
        "should not close before a digit"
    );

    assert_eq!(
        to_html_with_options("$x$y", &compmd)?,
        "<p><code class=\"language-math math-inline\">x</code>y</p>",
        "should close before a letter"
    );

    assert_eq!(
        to_html_with_options("$x{y$ z}$", &compmd)?,
        "<p><code class=\"language-math math-inline\">x{y$ z}</code></p>",
        "should not close in a group"
    );

    assert_eq!(
        to_html_with_options("$x{y $ z}$", &compmd)?,
        "<p><code class=\"language-math math-inline\">x{y $ z}</code></p>",
        "should not end the attempt at a `$` after white space in a group"
    );

    assert_eq!(
        to_html_with_options("$a}b$", &compmd)?,
        "<p><code class=\"language-math math-inline\">a}b</code></p>",
        "should support a closing brace with no group open"
    );

    assert_eq!(
        to_html_with_options("$a\\$b$ $c\\{d$", &compmd)?,
        "<p><code class=\"language-math math-inline\">a\\$b</code> <code class=\"language-math math-inline\">c\\{d</code></p>",
        "should support escapes in math, which don’t close or group"
    );

    assert_eq!(
        to_html_with_options("\\$a$", &compmd)?,
        "<p>$a$</p>",
        "should not support an escaped opening `$`"
    );

    assert_eq!(
        to_html_with_options("$$a$$", &compmd)?,
        "<p>$$a$$</p>",
        "should not support `$$` in text"
    );

    assert_eq!(
        to_html_with_options("$a\nb$", &compmd)?,
        "<p><code class=\"language-math math-inline\">a b</code></p>",
        "should support a line ending in math"
    );

    assert_eq!(
        to_html_with_options("| $a\\|b$ |\n| - |", &compmd)?,
        "<table>\n<thead>\n<tr>\n<th><code class=\"language-math math-inline\">a|b</code></th>\n</tr>\n</thead>\n</table>",
        "should support math in a table cell, `|` escaped"
    );

    assert_eq!(
        to_html_with_options(
            "costs $5 and $10, $x$",
            &Options {
                parse: ParseOptions {
                    constructs: Constructs::compmd(),
                    ..Default::default()
                },
                ..Default::default()
            }
        )?,
        "<p>costs $5 and $10, <code class=\"language-math math-inline\">x</code></p>",
        "should support math (text) w/ `Constructs::compmd()`"
    );

    assert_eq!(
        to_mdast("$x^2$", &compmd.parse)?,
        Node::Root(Root {
            children: vec![Node::Paragraph(Paragraph {
                children: vec![Node::InlineMath(InlineMath {
                    value: "x^2".into(),
                    position: Some(Position::new(1, 1, 0, 1, 6, 5))
                })],
                position: Some(Position::new(1, 1, 0, 1, 6, 5))
            })],
            position: Some(Position::new(1, 1, 0, 1, 6, 5))
        }),
        "should support math (text) as `InlineMath`s in mdast"
    );

    Ok(())
}
