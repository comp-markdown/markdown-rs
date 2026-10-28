//! CompMD: wiki link occurs in the [text][] content type.
//!
//! ## Grammar
//!
//! CompMD: wiki link forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! compmd_wiki_link ::= '[' '[' part [('|' | '\' '|') part] ']' ']'
//!
//! ; Restriction: no `[`, no `]` but the closing ones, no `|` in the label.
//! part ::= 1*line
//! ```
//!
//! In a GFM table, the separator is written `\|`, as GFM requires.
//! Wiki links change how existing text parses (`[[a]]` next to a definition of
//! `[a]` is a link in brackets in `CommonMark`), so a CompMD document turns
//! them on itself.
//! What the target points to is not decided here: the link is one token, and
//! a CompMD processor reads it.
//!
//! ## HTML
//!
//! A wiki link has no HTML equivalent here: it is written as its text, as
//! markdown without CompMD shows it.
//!
//! ## Tokens
//!
//! * [`CompmdWikiLink`][Name::CompmdWikiLink]
//!
//! ## References
//!
//! * [*§ 2.3 Base syntax* in the CompMD spec](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md#23-base-syntax)
//!
//! [text]: crate::construct::text

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// The byte after the current one.
fn next_byte(tokenizer: &Tokenizer) -> Option<u8> {
    tokenizer
        .parse_state
        .bytes
        .get(tokenizer.point.index + 1)
        .copied()
}

/// Start of a wiki link.
///
/// ```markdown
/// > | a [[b|c]] d
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.options.constructs.compmd_wiki_link
        && tokenizer.current == Some(b'[')
        && next_byte(tokenizer) == Some(b'[')
    {
        tokenizer.enter(Name::CompmdWikiLink);
        tokenizer.consume();
        State::Next(StateName::CompmdWikiLinkOpen)
    } else {
        State::Nok
    }
}

/// At the second `[`.
///
/// ```markdown
/// > | a [[b|c]] d
///        ^
/// ```
pub fn open(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    State::Next(StateName::CompmdWikiLinkPartStart)
}

/// At the start of the target or the label: neither is empty.
///
/// ```markdown
/// > | a [[b|c]] d
///         ^ ^
/// ```
pub fn part_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n' | b'[' | b']' | b'|') => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
        _ => State::Retry(StateName::CompmdWikiLinkPart),
    }
}

/// In the target (`marker` is `0`) or the label (`marker` is `|`).
///
/// ```markdown
/// > | a [[b|c]] d
///         ^ ^
/// ```
pub fn part(tokenizer: &mut Tokenizer) -> State {
    let in_target = tokenizer.tokenize_state.marker == 0;

    match tokenizer.current {
        Some(b']') if next_byte(tokenizer) == Some(b']') => {
            tokenizer.tokenize_state.marker = 0;
            tokenizer.consume();
            State::Next(StateName::CompmdWikiLinkClose)
        }
        Some(b'|') if in_target => {
            tokenizer.tokenize_state.marker = b'|';
            tokenizer.consume();
            State::Next(StateName::CompmdWikiLinkPartStart)
        }
        // In a GFM table, the separator is written `\|`.
        Some(b'\\') if in_target && next_byte(tokenizer) == Some(b'|') => {
            tokenizer.tokenize_state.marker = b'|';
            tokenizer.consume();
            State::Next(StateName::CompmdWikiLinkSeparator)
        }
        None | Some(b'\n' | b'[' | b']' | b'|') => {
            tokenizer.tokenize_state.marker = 0;
            State::Nok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdWikiLinkPart)
        }
    }
}

/// At the `|` of an escaped separator.
///
/// ```markdown
/// > | | [[b\|c]] |
///            ^
/// ```
pub fn separator(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    State::Next(StateName::CompmdWikiLinkPartStart)
}

/// At the second `]`.
///
/// ```markdown
/// > | a [[b|c]] d
///              ^
/// ```
pub fn close(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    tokenizer.exit(Name::CompmdWikiLink);
    State::Ok
}
