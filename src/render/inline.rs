//! What is drawn for each piece of a line, shared by every output format.

use crate::model::{Content, Inline};
use std::borrow::Cow;

/// The characters drawn for `inline`. A chord is drawn as its symbol, e.g.
/// `Bbm7/F`.
pub(super) fn drawn_text(inline: &Inline) -> Cow<'_, str> {
    match &inline.content {
        Content::Text(text) => Cow::Borrowed(text.as_ref()),
        Content::Chord(chord) => Cow::Owned(chord.to_string()),
    }
}
