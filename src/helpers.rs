//! Some utilities.
use std::time::Duration;

/// Convert a vector of items into a string.
///
/// The items should support `.to_string()`.
#[macro_export]
macro_rules! format_vec {
    ($items:ident) => {
        format_vec!($items, " ")
    };
    ($items:ident, $joiner:expr) => {
        $items
            .iter()
            .map(|it| it.to_string())
            .collect::<Vec<_>>()
            .join($joiner)
    };
}

/// A little custom trait to make it more convenient to work with Durations.
pub trait IntoDuration {
    fn into_duration(self) -> Duration;
}

impl IntoDuration for Duration {
    fn into_duration(self) -> Duration {
        self
    }
}

impl IntoDuration for u64 {
    fn into_duration(self) -> Duration {
        Duration::from_millis(self)
    }
}
