//! Top-level save layout detection and orchestration.
//!
//! This module keeps the public `format` surface small while splitting
//! implementation by responsibility.

mod compatibility;
mod decode;
mod edition_hint;
mod encode;
mod layout;
mod summary;

pub use layout::detect_format;
pub use edition_hint::detect_edition_hint;
pub use layout::FormatId;
pub use layout::Layout;
pub use layout::LayoutV99;
pub use layout::LayoutV105;

pub(crate) use compatibility::compatibility_issues;
pub(crate) use decode::decode;
pub(crate) use encode::encode;
pub(crate) use summary::summarize;

#[cfg(test)]
use layout::CHARACTER_SECTION_START;

#[cfg(test)]
mod tests;
