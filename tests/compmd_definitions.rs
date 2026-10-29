use markdown::{mdast::Node, message, to_html_with_options, to_mdast, Options, ParseOptions};
use pretty_assertions::assert_eq;

/// The kinds of the first paragraph's children.
fn kinds(value: &str, options: &ParseOptions) -> Result<Vec<&'static str>, message::Message> {
    let tree = to_mdast(value, options)?;
    let paragraph = &tree.children().unwrap()[0];
    Ok(paragraph
        .children()
        .unwrap()
        .iter()
        .map(|node| match node {
            Node::LinkReference(_) => "linkReference",
            Node::ImageReference(_) => "imageReference",
            Node::FootnoteReference(_) => "footnoteReference",
            Node::Text(_) => "text",
            _ => "other",
        })
        .collect())
}

#[test]
fn compmd_definitions() -> Result<(), message::Message> {
    let outside = ParseOptions {
        definitions: vec!["docs".into()],
        gfm_footnote_definitions: vec!["1".into()],
        ..ParseOptions::gfm()
    };

    assert_eq!(
        kinds("[the docs][docs]", &ParseOptions::gfm())?,
        vec!["text"],
        "should not see a reference as a link without its definition"
    );

    assert_eq!(
        kinds("[the docs][docs] [docs][] [docs] ![logo][docs]", &outside)?,
        vec![
            "linkReference",
            "text",
            "linkReference",
            "text",
            "linkReference",
            "text",
            "imageReference"
        ],
        "should see full, collapsed and shortcut references to a label defined outside"
    );

    assert_eq!(
        kinds("[x][ DOCS\t]", &outside)?,
        vec!["linkReference"],
        "should compare labels as references do"
    );

    let identifier = ParseOptions {
        definitions: vec!["a  b".into()],
        ..ParseOptions::gfm()
    };
    assert_eq!(
        kinds("[x][A b]", &identifier)?,
        vec!["linkReference"],
        "should normalize the labels given"
    );

    assert_eq!(
        kinds("[x][other]", &outside)?,
        vec!["text"],
        "should not see other labels as defined"
    );

    assert_eq!(
        kinds("a[^1]", &outside)?,
        vec!["text", "footnoteReference"],
        "should see a footnote call to a label defined outside"
    );

    assert_eq!(
        kinds("a[^2]", &outside)?,
        vec!["text"],
        "should not see other footnote labels as defined"
    );

    assert_eq!(
        kinds("[x][docs]\n\n[docs]: /x", &outside)?,
        vec!["linkReference"],
        "should allow a label defined both outside and in the text"
    );

    assert_eq!(
        to_html_with_options(
            "[the docs][docs] a[^1]",
            &Options {
                parse: ParseOptions {
                    definitions: vec!["docs".into()],
                    gfm_footnote_definitions: vec!["1".into()],
                    ..ParseOptions::gfm()
                },
                ..Options::gfm()
            }
        )?,
        "<p>[the docs][docs] a[^1]</p>",
        "should ignore labels defined outside in HTML"
    );

    Ok(())
}
