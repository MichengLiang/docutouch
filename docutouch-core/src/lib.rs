pub mod fs_tools;
pub mod patch_presentation;
pub mod patch_runtime;
pub mod path_display;
mod presentation_shared;
pub mod search_text;
pub mod splice_presentation;
pub mod splice_program;
pub mod splice_runtime;
pub mod splice_selection;

pub use codex_apply_patch::AffectedPaths;
pub use fs_tools::{
    DirectoryListOptions, DirectoryListResult, ReadFileLineRange, ReadFileOptions, ReadFileResult,
    ReadFileSampledViewOptions, TimestampField, list_directory, normalize_sampled_view_options,
    parse_read_file_line_range_text, read_file, read_file_with_sampled_view,
};
pub use patch_presentation::{PatchPresentationContext, format_patch_outcome};
pub use patch_runtime::{
    ApplyOutcome, FailureDetails, FailurePayload, PatchSourceKind, PatchSourceReference,
    PatchWorkspaceRequirement, apply_patch_program, apply_patch_program_with_source,
    extract_patch_paths, formal_patch_path, patch_workspace_requirement,
};
pub use path_display::{display_path, format_scope};
pub use search_text::{SearchTextView, search_text};
pub use splice_presentation::{SplicePresentationContext, format_splice_result};
pub use splice_program::{
    SpliceAction, SpliceProgram, SpliceProgramParseError, TargetAction, TransferSourceKind,
    extract_splice_paths, parse_splice_program,
};
pub use splice_runtime::{
    SpliceDiagnosticTargetAnchor, SpliceFailedUnit, SpliceFailureDetails, SpliceRuntimeError,
    SpliceRuntimeOutcome, SpliceWorkspaceRequirement, apply_splice_program,
    splice_workspace_requirement,
};
pub use splice_selection::{
    ResolvedSelection, SelectionBlock, SelectionDiagnosticCode, SelectionItem, SelectionLine,
    SelectionParseError, SelectionResolveError, SelectionSide, parse_selection_block,
    resolve_selection_block,
};
