//! Helper macro.

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
