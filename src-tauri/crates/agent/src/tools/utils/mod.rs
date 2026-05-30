//! `agent/tools/utils/` — shared helpers.

pub mod diff;
pub mod path;
pub mod truncate;

pub use diff::{
    detect_line_ending, fuzzy_find_text, generate_diff_string, normalize_for_fuzzy_match,
    normalize_to_lf, restore_line_endings, strip_bom, DiffOutput, FuzzyMatchResult,
};
pub use path::{
    cow_config_dir, cow_env_file, expand_path, is_cow_config_dir, is_cow_env_file, resolve_path,
};
pub use truncate::{
    format_size, truncate_head, truncate_line, truncate_tail, TruncationResult, DEFAULT_MAX_BYTES,
    DEFAULT_MAX_LINES, GREP_MAX_LINE_LENGTH,
};
