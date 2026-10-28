//! CompMD: math (text), by Pandoc’s rule, occurs in the [text][] content
//! type.
//!
//! ## Grammar
//!
//! CompMD: math (text) forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! ; Restriction: the opening `$` is not after or before another `$`.
//! ; Restriction: `content` doesn’t start with whitespace.
//! ; Restriction: the closing `$` is not after whitespace, not before an ASCII
//! ; digit, and outside braces.
//! ; Restriction: a `$` after whitespace, outside braces, ends the attempt.
//! compmd_math_text ::= '$' content '$'
//!
//! ; Restriction: an unescaped `{` opens a group, `}` closes one.
//! content ::= 1*(byte | '\' byte)
//! ```
//!
//! This is Pandoc’s `tex_math_dollars`, in full, so dollar amounts stay text:
//! in `It costs $5 and $6. Math $x^2$`, only `$x^2$` is math.
//! It differs from [math (text)][crate::construct::raw_text], which takes
//! `$` sequences like code (text) takes backticks.
//! The events are those of math (text), so they compile the same.
//!
//! ## HTML
//!
//! As [math (text)][crate::construct::raw_text]:
//! `<code class="language-math math-inline">`.
//!
//! ## Tokens
//!
//! * [`MathText`][Name::MathText]
//! * [`MathTextData`][Name::MathTextData]
//! * [`MathTextSequence`][Name::MathTextSequence]
//! * [`LineEnding`][Name::LineEnding]
//!
//! ## References
//!
//! * [*§ 2.3 Base syntax* in the CompMD spec](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md#23-base-syntax)
//! * [`tex_math_dollars` in Pandoc](https://pandoc.org/MANUAL.html#extension-tex_math_dollars)
//!
//! [text]: crate::construct::text

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;

/// Reset the state used by this construct.
fn reset(tokenizer: &mut Tokenizer) {
    tokenizer.tokenize_state.seen = false;
    tokenizer.tokenize_state.size = 0;
}

/// Close math data, if open.
fn close_data(tokenizer: &mut Tokenizer) {
    if tokenizer.tokenize_state.seen {
        tokenizer.exit(Name::MathTextData);
        tokenizer.tokenize_state.seen = false;
    }
}

/// Start of math (text).
///
/// ```markdown
/// > | a $b$ c
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.options.constructs.compmd_math_text
        && tokenizer.current == Some(b'$')
        && tokenizer.previous != Some(b'$')
    {
        tokenizer.enter(Name::MathText);
        tokenizer.enter(Name::MathTextSequence);
        tokenizer.consume();
        tokenizer.exit(Name::MathTextSequence);
        State::Next(StateName::CompmdMathTextOpen)
    } else {
        State::Nok
    }
}

/// After the opening `$`: math starts with a character that is not
/// whitespace or `$`.
///
/// ```markdown
/// > | a $b$ c
///        ^
/// ```
pub fn open(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\t' | b'\n' | b' ' | b'$') => State::Nok,
        _ => State::Retry(StateName::CompmdMathTextInside),
    }
}

/// In math.
///
/// ```markdown
/// > | a $b$ c
///        ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'\n') => {
            close_data(tokenizer);
            tokenizer.enter(Name::LineEnding);
            tokenizer.consume();
            tokenizer.exit(Name::LineEnding);
            State::Next(StateName::CompmdMathTextInside)
        }
        Some(b'$') if tokenizer.tokenize_state.size == 0 => {
            if matches!(tokenizer.previous, Some(b'\t' | b'\n' | b' ')) {
                // A `$` after whitespace ends the attempt.
                reset(tokenizer);
                State::Nok
            } else if tokenizer
                .parse_state
                .bytes
                .get(tokenizer.point.index + 1)
                .map_or(false, u8::is_ascii_digit)
            {
                State::Retry(StateName::CompmdMathTextContent)
            } else {
                close_data(tokenizer);
                reset(tokenizer);
                tokenizer.enter(Name::MathTextSequence);
                tokenizer.consume();
                tokenizer.exit(Name::MathTextSequence);
                tokenizer.exit(Name::MathText);
                State::Ok
            }
        }
        _ => State::Retry(StateName::CompmdMathTextContent),
    }
}

/// At a byte of content.
///
/// ```markdown
/// > | a ${b}$ c
///        ^^^
/// ```
pub fn content(tokenizer: &mut Tokenizer) -> State {
    if !tokenizer.tokenize_state.seen {
        tokenizer.enter(Name::MathTextData);
        tokenizer.tokenize_state.seen = true;
    }

    match tokenizer.current {
        Some(b'{') => tokenizer.tokenize_state.size += 1,
        Some(b'}') if tokenizer.tokenize_state.size > 0 => tokenizer.tokenize_state.size -= 1,
        _ => {}
    }

    let escape = tokenizer.current == Some(b'\\');
    tokenizer.consume();

    if escape {
        State::Next(StateName::CompmdMathTextEscape)
    } else {
        State::Next(StateName::CompmdMathTextInside)
    }
}

/// After `\` in math: the next character is content, whatever it is.
///
/// ```markdown
/// > | a $b\$c$ d
///          ^
/// ```
pub fn escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => State::Retry(StateName::CompmdMathTextInside),
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdMathTextInside)
        }
    }
}
