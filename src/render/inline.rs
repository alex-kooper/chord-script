//! Inlines as plain text, shared by every output format.

use crate::model::{Content, Inline};
use std::borrow::Cow;

/// `inline` as it was typed, for messages: a chord is its ASCII symbol, e.g.
/// `Bbm7/F`, not the ♭ and raised or lowered parts it is drawn with.
pub(super) fn typed_text(inline: &Inline) -> Cow<'_, str> {
    match &inline.content {
        Content::Text(text) => Cow::Borrowed(text.as_ref()),
        Content::Chord(chord) => Cow::Owned(chord.to_string()),
    }
}
