//! CompMD: inline component occurs in the [text][] content type.
//!
//! ## Grammar
//!
//! CompMD: inline component forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! ; Restriction: the `:` is at the start, or after whitespace or punctuation
//! ; other than `:` and `\`.
//! ; Restriction: brackets in `fallback` must be balanced; escaped brackets and
//! ; brackets in code (text) don’t count.
//! ; Restriction: `params` must be on one line; when they aren’t, the component
//! ; ends at its `]`.
//! compmd_component_inline ::= ':' name '[' fallback ']' [params]
//!
//! name ::= ascii_alphabetic *(ascii_alphanumeric | '-' | '_')
//! fallback ::= *byte
//! ; Restriction: not followed by another `{`, which starts an interpolation.
//! params ::= '{' *(string | expression | json | line) '}'
//! string ::= '"' *(line | '\' line) '"'
//! ; Restriction: parentheses are counted outside CEL string literals
//! ; (`"…"`, `'…'`, triple quoted, raw with `r`).
//! expression ::= '(' *line ')'
//! ; Restriction: brackets and braces are counted outside JSON strings.
//! json ::= ('[' *line ']') | ('{' *line '}')
//! ```
//!
//! What the fallback and params mean is not decided here: the component is
//! one token, and a CompMD processor reads it.
//!
//! ## HTML
//!
//! CompMD’s components have no HTML equivalent here: the component is written
//! as its text, as markdown without CompMD shows it.
//!
//! ## Tokens
//!
//! * [`CompmdComponentInline`][Name::CompmdComponentInline]
//!
//! ## References
//!
//! * [*§ 3 Syntax* in the CompMD spec](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md#3-syntax)
//!
//! [text]: crate::construct::text

use crate::event::Name;
use crate::state::{Name as StateName, State};
use crate::tokenizer::Tokenizer;
use crate::util::char::{before_index, classify_opt, Kind};

/// The byte after the current one.
fn next_byte(tokenizer: &Tokenizer) -> Option<u8> {
    tokenizer
        .parse_state
        .bytes
        .get(tokenizer.point.index + 1)
        .copied()
}

/// Reset the state used by this construct.
fn reset(tokenizer: &mut Tokenizer) {
    tokenizer.tokenize_state.marker = 0;
    tokenizer.tokenize_state.marker_b = 0;
    tokenizer.tokenize_state.seen = false;
    tokenizer.tokenize_state.size = 0;
    tokenizer.tokenize_state.size_b = 0;
    tokenizer.tokenize_state.size_c = 0;
}

/// Start of an inline component.
///
/// ```markdown
/// > | a :b[c]{d} e
///       ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer
        .parse_state
        .options
        .constructs
        .compmd_component_inline
        && tokenizer.current == Some(b':')
    {
        let before = before_index(tokenizer.parse_state.bytes, tokenizer.point.index);
        let allowed = match classify_opt(before) {
            Kind::Whitespace => true,
            Kind::Punctuation => before != Some(':') && before != Some('\\'),
            Kind::Other => false,
        };

        if allowed {
            tokenizer.enter(Name::CompmdComponentInline);
            tokenizer.consume();
            return State::Next(StateName::CompmdComponentInlineNameStart);
        }
    }

    State::Nok
}

/// After `:`, at the name.
///
/// ```markdown
/// > | a :b[c]{d} e
///        ^
/// ```
pub fn name_start(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(byte) if byte.is_ascii_alphabetic() => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineName)
        }
        _ => State::Nok,
    }
}

/// In the name.
///
/// ```markdown
/// > | a :b[c]{d} e
///         ^
/// ```
pub fn name(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineFallback)
        }
        Some(byte) if byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_' => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineName)
        }
        _ => State::Nok,
    }
}

/// In the fallback.
///
/// ```markdown
/// > | a :b[c]{d} e
///          ^
/// ```
pub fn fallback(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'[') => {
            tokenizer.tokenize_state.size += 1;
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineFallback)
        }
        Some(b']') if tokenizer.tokenize_state.size == 0 => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineAfterFallback)
        }
        Some(b']') => {
            tokenizer.tokenize_state.size -= 1;
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineFallback)
        }
        Some(b'\\') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineFallbackEscape)
        }
        // A code (text) binds tighter than the brackets: when a run of the
        // same size closes it, skip to its end; otherwise the run is text.
        Some(b'`') => {
            tokenizer.attempt(
                State::Next(StateName::CompmdComponentInlineFallback),
                State::Next(StateName::CompmdComponentInlineFallbackRun),
            );
            State::Retry(StateName::CompmdComponentInlineCodeOpen)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineFallback)
        }
    }
}

/// After `\` in the fallback: brackets, backslashes, and backticks are
/// escaped.
///
/// ```markdown
/// > | a :b[c\]]{d} e
///            ^
/// ```
pub fn fallback_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'[' | b']' | b'\\' | b'`') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineFallback)
        }
        _ => State::Retry(StateName::CompmdComponentInlineFallback),
    }
}

/// At a run of backticks in the fallback that opens no code (text): text.
///
/// ```markdown
/// > | a :b[c`]{d} e
///           ^
/// ```
pub fn fallback_run(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'`') {
        tokenizer.consume();
        State::Next(StateName::CompmdComponentInlineFallbackRun)
    } else {
        State::Retry(StateName::CompmdComponentInlineFallback)
    }
}

/// In the opening run of a code (text) in the fallback.
///
/// ```markdown
/// > | a :b[`c]`]{d} e
///          ^
/// ```
pub fn code_open(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'`') {
        tokenizer.tokenize_state.size_c += 1;
        tokenizer.consume();
        State::Next(StateName::CompmdComponentInlineCodeOpen)
    } else {
        State::Retry(StateName::CompmdComponentInlineCodeInside)
    }
}

/// In a code (text) in the fallback.
///
/// ```markdown
/// > | a :b[`c]`]{d} e
///           ^^
/// ```
pub fn code_inside(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None => {
            tokenizer.tokenize_state.size_b = 0;
            tokenizer.tokenize_state.size_c = 0;
            State::Nok
        }
        Some(b'`') => {
            tokenizer.tokenize_state.size_b = 0;
            State::Retry(StateName::CompmdComponentInlineCodeClose)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineCodeInside)
        }
    }
}

/// In a run of backticks in a code (text) in the fallback: it closes the code
/// when it is as long as the opening run.
///
/// ```markdown
/// > | a :b[`c]`]{d} e
///             ^
/// ```
pub fn code_close(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(b'`') {
        tokenizer.tokenize_state.size_b += 1;
        tokenizer.consume();
        State::Next(StateName::CompmdComponentInlineCodeClose)
    } else if tokenizer.tokenize_state.size_b == tokenizer.tokenize_state.size_c {
        tokenizer.tokenize_state.size_b = 0;
        tokenizer.tokenize_state.size_c = 0;
        State::Ok
    } else {
        State::Retry(StateName::CompmdComponentInlineCodeInside)
    }
}

/// After the fallback, maybe at params.
///
/// ```markdown
/// > | a :b[c]{d} e
///            ^
/// ```
pub fn after_fallback(tokenizer: &mut Tokenizer) -> State {
    tokenizer.tokenize_state.size = 0;

    if tokenizer.current == Some(b'{') && next_byte(tokenizer) != Some(b'{') {
        tokenizer.attempt(
            State::Next(StateName::CompmdComponentInlineEnd),
            State::Next(StateName::CompmdComponentInlineEnd),
        );
        State::Retry(StateName::CompmdComponentInlineParamsOpen)
    } else {
        State::Retry(StateName::CompmdComponentInlineEnd)
    }
}

/// At the `{` of params.
///
/// ```markdown
/// > | a :b[c]{d} e
///            ^
/// ```
pub fn params_open(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    State::Next(StateName::CompmdComponentInlineParams)
}

/// End of an inline component.
///
/// ```markdown
/// > | a :b[c]{d} e
///               ^
/// ```
pub fn end(tokenizer: &mut Tokenizer) -> State {
    reset(tokenizer);
    tokenizer.exit(Name::CompmdComponentInline);
    State::Ok
}

/// In params, between values.
///
/// ```markdown
/// > | a :b[c]{d} e
///             ^
/// ```
pub fn params(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'}') => {
            tokenizer.consume();
            State::Ok
        }
        Some(b'"') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsString)
        }
        Some(b'(') => {
            tokenizer.tokenize_state.size_b = 1;
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsExpression)
        }
        Some(b'[' | b'{') => {
            tokenizer.tokenize_state.size_b = 1;
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJson)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParams)
        }
    }
}

/// In a string value.
///
/// ```markdown
/// > | a :b[c]{d="e"} f
///                ^
/// ```
pub fn params_string(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'"') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParams)
        }
        Some(b'\\') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsStringEscape)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsString)
        }
    }
}

/// After `\` in a string value.
///
/// ```markdown
/// > | a :b[c]{d="\""} f
///                 ^
/// ```
pub fn params_string_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsString)
        }
    }
}

/// In an expression value.
///
/// ```markdown
/// > | a :b[c]{d=(e(f))} g
///                ^
/// ```
pub fn params_expression(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        Some(quote @ (b'"' | b'\'')) => {
            tokenizer.tokenize_state.marker = quote;
            tokenizer.tokenize_state.marker_b =
                u8::from(matches!(tokenizer.previous, Some(b'r' | b'R')));
            let triple = tokenizer
                .parse_state
                .bytes
                .get(tokenizer.point.index + 1..tokenizer.point.index + 3)
                == Some(&[quote, quote][..]);
            tokenizer.tokenize_state.seen = triple;
            tokenizer.tokenize_state.size_c = if triple { 3 } else { 1 };
            State::Retry(StateName::CompmdComponentInlineParamsExpressionQuote)
        }
        Some(b'(') => {
            tokenizer.tokenize_state.size_b += 1;
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsExpression)
        }
        Some(b')') => {
            tokenizer.tokenize_state.size_b -= 1;
            tokenizer.consume();
            if tokenizer.tokenize_state.size_b == 0 {
                State::Next(StateName::CompmdComponentInlineParams)
            } else {
                State::Next(StateName::CompmdComponentInlineParamsExpression)
            }
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsExpression)
        }
    }
}

/// In the quotes that open or close a string in an expression: `size_c` of
/// them, three for a triple quoted string.
///
/// ```markdown
/// > | a :b[c]{d=('''e''')} f
///                ^^^   ^^^
/// ```
pub fn params_expression_quote(tokenizer: &mut Tokenizer) -> State {
    tokenizer.consume();
    tokenizer.tokenize_state.size_c -= 1;

    if tokenizer.tokenize_state.size_c > 0 {
        State::Next(StateName::CompmdComponentInlineParamsExpressionQuote)
    } else if tokenizer.tokenize_state.marker == 0 {
        State::Next(StateName::CompmdComponentInlineParamsExpression)
    } else {
        State::Next(StateName::CompmdComponentInlineParamsExpressionString)
    }
}

/// In a string in an expression value.
///
/// ```markdown
/// > | a :b[c]{d=("e)")} f
///                 ^^
/// ```
pub fn params_expression_string(tokenizer: &mut Tokenizer) -> State {
    let quote = tokenizer.tokenize_state.marker;

    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'\\') if tokenizer.tokenize_state.marker_b == 0 => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsExpressionEscape)
        }
        Some(byte)
            if byte == quote
                && (!tokenizer.tokenize_state.seen
                    || tokenizer
                        .parse_state
                        .bytes
                        .get(tokenizer.point.index + 1..tokenizer.point.index + 3)
                        == Some(&[quote, quote][..])) =>
        {
            tokenizer.tokenize_state.size_c = if tokenizer.tokenize_state.seen { 3 } else { 1 };
            tokenizer.tokenize_state.marker = 0;
            tokenizer.tokenize_state.marker_b = 0;
            tokenizer.tokenize_state.seen = false;
            State::Retry(StateName::CompmdComponentInlineParamsExpressionQuote)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsExpressionString)
        }
    }
}

/// After `\` in a string in an expression value.
///
/// ```markdown
/// > | a :b[c]{d=("\"")} f
///                  ^
/// ```
pub fn params_expression_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsExpressionString)
        }
    }
}

/// In a JSON value.
///
/// ```markdown
/// > | a :b[c]{d=[1, [2]]} e
///                ^
/// ```
pub fn params_json(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'"') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJsonString)
        }
        Some(b'[' | b'{') => {
            tokenizer.tokenize_state.size_b += 1;
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJson)
        }
        Some(b']' | b'}') => {
            tokenizer.tokenize_state.size_b -= 1;
            tokenizer.consume();
            if tokenizer.tokenize_state.size_b == 0 {
                State::Next(StateName::CompmdComponentInlineParams)
            } else {
                State::Next(StateName::CompmdComponentInlineParamsJson)
            }
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJson)
        }
    }
}

/// In a string in a JSON value.
///
/// ```markdown
/// > | a :b[c]{d=["]"]} e
///                 ^
/// ```
pub fn params_json_string(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        Some(b'"') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJson)
        }
        Some(b'\\') => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJsonEscape)
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJsonString)
        }
    }
}

/// After `\` in a string in a JSON value.
///
/// ```markdown
/// > | a :b[c]{d=["\""]} e
///                  ^
/// ```
pub fn params_json_escape(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            reset(tokenizer);
            State::Nok
        }
        _ => {
            tokenizer.consume();
            State::Next(StateName::CompmdComponentInlineParamsJsonString)
        }
    }
}
