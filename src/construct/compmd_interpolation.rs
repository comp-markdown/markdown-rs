//! CompMD: interpolation occurs in the [text][] content type.
//!
//! ## Grammar
//!
//! CompMD: interpolation forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! compmd_interpolation ::= '{' '{' *space_or_tab path *space_or_tab '}' '}'
//! ; Restriction: only when no interpolation starts there.
//! compmd_interpolation_literal ::= '{' '{'
//!
//! path ::= name *('.' name)
//! name ::= (ascii_alphabetic | '_') *(ascii_alphanumeric | '_')
//! ```
//!
//! An interpolation is on one line, and it is recognized before attention
//! and links, as code (text) is: `*{{a*}}` is not an interpolation.
//! A `{{` that starts no interpolation, such as in `{{ 1 + 2 }}`, is a token
//! of its own, so a CompMD processor can report it.
//!
//! ## HTML
//!
//! An interpolation has no HTML equivalent here: it is written as its text,
//! as markdown without CompMD shows it.
//!
//! ## Tokens
//!
//! * [`CompmdInterpolation`][Name::CompmdInterpolation]
//! * [`CompmdInterpolationLiteral`][Name::CompmdInterpolationLiteral]
//!
//! ## References
//!
//! * [*§ 8.1 Interpolation* in the CompMD spec](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md#81-interpolation)
//!
//! [text]: crate::construct::text

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Whether `{{` starts here, and the construct is on.
fn at_open(tokenizer: &Tokenizer) -> bool {
    tokenizer
        .parse_state
        .options
        .constructs
        .compmd_interpolation
        && tokenizer.current == Some(b'{')
        && tokenizer.parse_state.bytes.get(tokenizer.point.index + 1) == Some(&b'{')
}

/// Start of an interpolation.
///
/// ```markdown
/// > | a {{ b.c }} d
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if at_open(tokenizer) {
        tokenizer.enter(Name::CompmdInterpolation);
        tokenizer.consume();
        State::Next(StateName::CompmdInterpolationOpen)
    } else {
        State::Nok
    }
}

/// At the second `{`.
///
/// ```markdown
/// > | a {{ b.c }} d
///        ^
/// ```
pub fn open(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    State::Next(StateName::CompmdInterpolationBeforePath)
}

/// After `{{`, before the path.
///
/// ```markdown
/// > | a {{ b.c }} d
///         ^
/// ```
pub fn before_path(tokenizer: &mut Tokenizer) -> State {
    if matches!(tokenizer.current, Some(b'\t' | b' ')) {
        tokenizer.consume();
        State::Next(StateName::CompmdInterpolationBeforePath)
    } else {
        State::Retry(StateName::CompmdInterpolationNameStart)
    }
}

/// At a name in the path.
///
/// ```markdown
/// > | a {{ b.c }} d
///          ^ ^
/// ```
pub fn name_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte) if byte.is_ascii_alphabetic() || byte == b'_' => {
            tokenizer.consume();
            State::Next(StateName::CompmdInterpolationName)
        }
        _ => State::Nok,
    }
}

/// In a name in the path.
///
/// ```markdown
/// > | a {{ bc.d }} e
///           ^
/// ```
pub fn name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'.') => {
            tokenizer.consume();
            State::Next(StateName::CompmdInterpolationNameStart)
        }
        Some(byte) if byte.is_ascii_alphanumeric() || byte == b'_' => {
            tokenizer.consume();
            State::Next(StateName::CompmdInterpolationName)
        }
        _ => State::Retry(StateName::CompmdInterpolationAfterPath),
    }
}

/// After the path, before `}}`.
///
/// ```markdown
/// > | a {{ b.c }} d
///             ^
/// ```
pub fn after_path(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'\t' | b' ') => {
            tokenizer.consume();
            State::Next(StateName::CompmdInterpolationAfterPath)
        }
        Some(b'}') if tokenizer.parse_state.bytes.get(tokenizer.point.index + 1) == Some(&b'}') => {
            tokenizer.consume();
            State::Next(StateName::CompmdInterpolationClose)
        }
        _ => State::Nok,
    }
}

/// At the second `}`.
///
/// ```markdown
/// > | a {{ b.c }} d
///               ^
/// ```
pub fn close(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    tokenizer.exit(Name::CompmdInterpolation);
    State::Ok
}

/// Start of a `{{` that starts no interpolation.
///
/// ```markdown
/// > | a {{ 1 + 2 }} b
///       ^
/// ```
pub fn literal_start(tokenizer: &mut Tokenizer) -> State {
    if at_open(tokenizer) {
        tokenizer.enter(Name::CompmdInterpolationLiteral);
        tokenizer.consume();
        State::Next(StateName::CompmdInterpolationLiteralOpen)
    } else {
        State::Nok
    }
}

/// At the second `{` of a `{{` that starts no interpolation.
///
/// ```markdown
/// > | a {{ 1 + 2 }} b
///        ^
/// ```
pub fn literal_open(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    tokenizer.exit(Name::CompmdInterpolationLiteral);
    State::Ok
}
