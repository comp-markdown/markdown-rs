//! Heading (setext) occurs in the [flow][] content type.
//!
//! ## Grammar
//!
//! Heading (setext) forms with the following BNF
//! (<small>see [construct][crate::construct] for character groups</small>):
//!
//! ```bnf
//! heading_setext ::= paragraph eol *space_or_tab (1*'-' | 1*'=') *space_or_tab
//!
//! ; See the `paragraph` construct for the BNF of that part.
//! ```
//!
//! As this construct occurs in flow, like all flow constructs, it must be
//! followed by an eol (line ending) or eof (end of file).
//!
//! See [`paragraph`][paragraph] for grammar, notes, and recommendations on
//! that part.
//!
//! In markdown, it is also possible to create headings with a
//! [heading (atx)][heading_atx] construct.
//! The benefit of setext headings is that their text can include line endings,
//! and by extensions also hard breaks (e.g., with
//! [hard break (escape)][hard_break_escape]).
//! However, their limit is that they cannot form `<h3>` through `<h6>`
//! headings.
//!
//! [Thematic breaks][thematic_break] formed with dashes and without whitespace
//! could be interpreted as a heading (setext).
//! Which one forms depends on whether there is text directly in fron of the
//! sequence.
//!
//! > 🏛 **Background**: the word *setext* originates from a small markup
//! > language by Ian Feldman from 1991.
//! > See [*§ Setext* on Wikipedia][wiki_setext] for more info.
//! > The word *atx* originates from a tiny markup language by Aaron Swartz
//! > from 2002.
//! > See [*§ atx, the true structured text format* on `aaronsw.com`][atx] for
//! > more info.
//!
//! ## HTML
//!
//! Heading (setext) in markdown relates to the `<h1>` and `<h2>` elements in
//! HTML.
//! See [*§ 4.3.6 The `h1`, `h2`, `h3`, `h4`, `h5`, and `h6` elements* in the
//! HTML spec][html] for more info.
//!
//! ## Recommendation
//!
//! Always use heading (atx), never heading (setext).
//!
//! ## Tokens
//!
//! * [`HeadingSetext`][Name::HeadingSetext]
//! * [`HeadingSetextText`][Name::HeadingSetextText]
//! * [`HeadingSetextUnderline`][Name::HeadingSetextUnderline]
//! * [`HeadingSetextUnderlineSequence`][Name::HeadingSetextUnderlineSequence]
//!
//! ## References
//!
//! * [`setext-underline.js` in `micromark`](https://github.com/micromark/micromark/blob/main/packages/micromark-core-commonmark/dev/lib/setext-underline.js)
//! * [*§ 4.3 Setext headings* in `CommonMark`](https://spec.commonmark.org/0.31/#setext-headings)
//!
//! [flow]: crate::construct::flow
//! [paragraph]: crate::construct::paragraph
//! [heading_atx]: crate::construct::heading_atx
//! [thematic_break]: crate::construct::thematic_break
//! [hard_break_escape]: crate::construct::hard_break_escape
//! [html]: https://html.spec.whatwg.org/multipage/sections.html#the-h1,-h2,-h3,-h4,-h5,-and-h6-elements
//! [wiki_setext]: https://en.wikipedia.org/wiki/Setext
//! [atx]: http://www.aaronsw.com/2002/atx/

use crate::construct::partial_space_or_tab::{space_or_tab, space_or_tab_min_max};
use crate::event::{Content, Event, Kind, Link, Name};
use crate::resolve::Name as ResolveName;
use crate::state::{Name as StateName, State};
use crate::subtokenize::Subresult;
use crate::tokenizer::Tokenizer;
use crate::util::{constant::TAB_SIZE, skip};
use alloc::{vec, vec::Vec};

/// At start of heading (setext) underline.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn start(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.parse_state.options.constructs.heading_setext
        && !tokenizer.lazy
        && !tokenizer.pierce
        && paragraph_before(tokenizer)
    {
        tokenizer.enter(Name::HeadingSetextUnderline);

        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(StateName::HeadingSetextBefore), State::Nok);
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
            State::Retry(StateName::HeadingSetextBefore)
        }
    } else {
        State::Nok
    }
}

/// Whether the line before is content that holds paragraph text.
///
/// Content holding only definitions has no paragraph to underline, so the
/// line is something else: a thematic break, or more content (as in
/// `micromark`).
/// Content is split into definitions and paragraphs only once the document
/// is parsed, so its lines are parsed here on their own to find out.
fn paragraph_before(tokenizer: &Tokenizer) -> bool {
    if tokenizer.events.is_empty() {
        return false;
    }

    let exit = skip::opt_back(
        &tokenizer.events,
        tokenizer.events.len() - 1,
        &[Name::LineEnding, Name::SpaceOrTab],
    );

    if tokenizer.events[exit].name != Name::Content {
        return false;
    }

    let chunks = content_chunks(&tokenizer.events, exit);
    let mut subtokenizer = Tokenizer::new(
        tokenizer.events[chunks[0]].point.clone(),
        tokenizer.parse_state,
    );
    let mut state = State::Next(StateName::ContentDefinitionBefore);

    for (index, &enter) in chunks.iter().enumerate() {
        let start = &tokenizer.events[enter].point;
        // A line before the last one ends after its line ending: `enter + 3`
        // is its `Exit:LineEnding`.
        let end = &tokenizer.events[if index + 1 < chunks.len() {
            enter + 3
        } else {
            enter + 1
        }]
        .point;
        if index > 0 {
            subtokenizer.define_skip(start.clone());
        }
        state = subtokenizer.push((start.index, start.vs), (end.index, end.vs), state);
    }

    // Content that fails to parse is left to the document’s own parse to
    // report: here it counts as a paragraph.
    subtokenizer.flush(state, false).is_err()
        || subtokenizer
            .events
            .iter()
            .any(|event| event.name == Name::Paragraph)
}

/// The `Enter:Content` of each line of the content whose last line exits at
/// `exit`, in order.
fn content_chunks(events: &[Event], exit: usize) -> Vec<usize> {
    let mut chunks = vec![exit - 1];

    while let Some(before) = chunks.last().and_then(|enter| enter.checked_sub(1)) {
        let previous = skip::opt_back(events, before, &[Name::SpaceOrTab]);
        if events[previous].name != Name::LineEnding || previous < 2 {
            break;
        }
        let previous_exit = previous - 2;
        if events[previous_exit].name != Name::Content {
            break;
        }
        chunks.push(previous_exit - 1);
    }

    chunks.reverse();
    chunks
}

/// After optional whitespace, at `-` or `=`.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn before(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        Some(b'-' | b'=') => {
            tokenizer.tokenize_state.marker = tokenizer.current.unwrap();
            tokenizer.enter(Name::HeadingSetextUnderlineSequence);
            State::Retry(StateName::HeadingSetextInside)
        }
        _ => State::Nok,
    }
}

/// In sequence.
///
/// ```markdown
///   | aa
/// > | ==
///     ^
/// ```
pub fn inside(tokenizer: &mut Tokenizer) -> State {
    if tokenizer.current == Some(tokenizer.tokenize_state.marker) {
        tokenizer.consume();
        State::Next(StateName::HeadingSetextInside)
    } else {
        tokenizer.tokenize_state.marker = 0;
        tokenizer.exit(Name::HeadingSetextUnderlineSequence);

        if matches!(tokenizer.current, Some(b'\t' | b' ')) {
            tokenizer.attempt(State::Next(StateName::HeadingSetextAfter), State::Nok);
            State::Retry(space_or_tab(tokenizer))
        } else {
            State::Retry(StateName::HeadingSetextAfter)
        }
    }
}

/// After sequence, after optional whitespace.
///
/// ```markdown
///   | aa
/// > | ==
///       ^
/// ```
pub fn after(tokenizer: &mut Tokenizer) -> State {
    match tokenizer.current {
        None | Some(b'\n') => {
            // Feel free to interrupt.
            tokenizer.interrupt = false;
            tokenizer.register_resolver(ResolveName::HeadingSetext);
            tokenizer.exit(Name::HeadingSetextUnderline);
            State::Ok
        }
        _ => State::Nok,
    }
}

/// Resolve heading (setext).
pub fn resolve(tokenizer: &mut Tokenizer) -> Option<Subresult> {
    let mut enter = skip::to(&tokenizer.events, 0, &[Name::HeadingSetextUnderline]);

    while enter < tokenizer.events.len() {
        let exit = skip::to(
            &tokenizer.events,
            enter + 1,
            &[Name::HeadingSetextUnderline],
        );

        // Find paragraph before
        let paragraph_exit_before = skip::opt_back(
            &tokenizer.events,
            enter - 1,
            &[Name::SpaceOrTab, Name::LineEnding, Name::BlockQuotePrefix],
        );

        // `start` takes an underline only after paragraph text.
        let paragraph = tokenizer.events[paragraph_exit_before].name == Name::Paragraph;
        debug_assert!(paragraph, "expected a paragraph before the underline");

        if paragraph {
            let paragraph_enter = skip::to_back(
                &tokenizer.events,
                paragraph_exit_before - 1,
                &[Name::Paragraph],
            );

            // Change types of Enter:Paragraph, Exit:Paragraph.
            tokenizer.events[paragraph_enter].name = Name::HeadingSetextText;
            tokenizer.events[paragraph_exit_before].name = Name::HeadingSetextText;

            // Add Enter:HeadingSetext, Exit:HeadingSetext.
            let mut heading_enter = tokenizer.events[paragraph_enter].clone();
            heading_enter.name = Name::HeadingSetext;
            tokenizer.map.add(paragraph_enter, 0, vec![heading_enter]);
            let mut heading_exit = tokenizer.events[exit].clone();
            heading_exit.name = Name::HeadingSetext;
            tokenizer.map.add(exit + 1, 0, vec![heading_exit]);
        } else {
            // Keep the line as paragraph text rather than lose it.
            tokenizer.events[enter].name = Name::Paragraph;
            tokenizer.events[exit].name = Name::Paragraph;
            // Replace what’s inside the underline (whitespace, sequence).
            tokenizer.map.add(
                enter + 1,
                exit - enter - 1,
                vec![
                    Event {
                        name: Name::Data,
                        kind: Kind::Enter,
                        point: tokenizer.events[enter].point.clone(),
                        link: Some(Link {
                            previous: None,
                            next: None,
                            content: Content::Text,
                        }),
                    },
                    Event {
                        name: Name::Data,
                        kind: Kind::Exit,
                        point: tokenizer.events[exit].point.clone(),
                        link: None,
                    },
                ],
            );
        }

        enter = skip::to(&tokenizer.events, exit + 1, &[Name::HeadingSetextUnderline]);
    }

    tokenizer.map.consume(&mut tokenizer.events);
    None
}
