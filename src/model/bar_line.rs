//! Bar lines: what stands at each end of a row and between its cells.

use derive_more::Display;

/// A bar line. It displays as written.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Display)]
pub enum BarLine {
    /// `|`
    #[default]
    #[display("|")]
    Single,
    /// `||`: a section starts or ends here
    #[display("||")]
    Double,
    /// `|||`: the piece ends here
    #[display("|||")]
    Final,
    /// `||:`: a repeat starts here
    #[display("||:")]
    RepeatStart,
    /// `:||`: a repeat ends here
    #[display(":||")]
    RepeatEnd,
    /// `:||:`: one repeat ends and the next starts here
    #[display(":||:")]
    RepeatBoth,
}
