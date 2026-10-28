//! CompMD: component line occurs in the [flow][] content type.
//!
//! ## Grammar
//!
//! CompMD: component line forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! compmd_component_line ::= *space_or_tab ':' ':' *line
//! ```
//!
//! As this construct occurs in flow, like all flow constructs, it must be
//! followed by an eol (line ending) or eof (end of file).
//!
//! What the line is (an opener such as `::: card {title="Hi"}`, a closer such
//! as `::: /card`, a slot such as `:: #footer`, a leaf such as
//! `:: chart[Sales]`, a continuation such as `::: + gap=m`, or a malformed
//! line) is not decided here: the line is one token, and a CompMD processor
//! reads it.
//! Like a thematic break, a component line can interrupt a paragraph, and it
//! ends a lazy line.
//!
//! ## HTML
//!
//! CompMD’s components have no HTML equivalent here: the line is written as
//! a paragraph of its text, as markdown without CompMD shows it.
//!
//! ## Tokens
//!
//! * [`CompmdComponentLine`][Name::CompmdComponentLine]
//! * [`SpaceOrTab`][Name::SpaceOrTab]
//!
//! ## References
//!
//! * [*§ 3 Syntax* in the CompMD spec](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md#3-syntax)
//!
//! [flow]: crate::construct::flow

use crate::construct::partial_space_or_tab::space_or_tab_min_max;
use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::constant::TAB_SIZE;

/// Start of a component line.
///
/// ```markdown
/// > | ::: card
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer
        .parse_state
        .options
        .constructs
        .compmd_component_line
    {
        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(
                State::Next(StateName::CompmdComponentLineBefore),
                State::Nok,
            );
            State::Retry(space_or_tab_min_max(
                tokenizer,
                0,
                if tokenizer.parse_state.options.constructs.code_indented {
                    TAB_SIZE - 1
                } else {
                    usize::MAX
                },
            ))
        } else {
            State::Retry(StateName::CompmdComponentLineBefore)
        }
    } else {
        State::Nok
    }
}

/// After optional whitespace, at the first `:`.
///
/// ```markdown
/// > | ::: card
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b':')
        && tokenizer.parse_state.bytes.get(tokenizer.point.index + 1) == Some(&b':')
    {
        tokenizer.enter(Name::CompmdComponentLine);
        tokenizer.consume();
        State::Next(StateName::CompmdComponentLineInside)
    } else {
        State::Nok
    }
}

/// In the line.
///
/// ```markdown
/// > | ::: card
///      ^^^^^^^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            tokenizer.exit(Name::CompmdComponentLine);
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            State::Ok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentLineInside)
        }
    }
}
