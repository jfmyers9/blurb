/// Shared types for the rich text editor abstraction.
/// Both TipTap (JS) and future native backends use these types.

#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    ToggleBold,
    ToggleItalic,
    ToggleStrike,
    SetHeading(u8),
    ToggleBulletList,
    ToggleOrderedList,
    ToggleBlockquote,
    ToggleCodeBlock,
    InsertHorizontalRule,
    ToggleLink(String),
    Undo,
    Redo,
}

#[derive(Default, Clone, PartialEq)]
pub struct ActiveState {
    pub bold: bool,
    pub italic: bool,
    pub strike: bool,
    pub h1: bool,
    pub h2: bool,
    pub h3: bool,
    pub bullet_list: bool,
    pub ordered_list: bool,
    pub blockquote: bool,
    pub code_block: bool,
}
