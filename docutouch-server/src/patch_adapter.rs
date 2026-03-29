use crate::transport_shell::{TransportInvocation, TransportSourceProvenance};
use docutouch_core::{
    PatchPresentationContext, apply_patch_program_with_source, format_patch_outcome,
};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

pub(crate) type PatchSourceProvenance = TransportSourceProvenance;

pub(crate) const APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV: &str =
    "DOCUTOUCH_APPLY_PATCH_NUMBERED_EVIDENCE_MODE";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PatchNumberedEvidenceMode {
    HeaderOnly,
    Full,
}

impl PatchNumberedEvidenceMode {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "header_only" => Ok(Self::HeaderOnly),
            "full" => Ok(Self::Full),
            _ => Err(format!(
                "--numbered-evidence-mode must be `header_only` or `full`, got `{value}`"
            )),
        }
    }

    fn as_env_value(self) -> &'static str {
        match self {
            Self::HeaderOnly => "header_only",
            Self::Full => "full",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PatchInvocationAdapter {
    transport: TransportInvocation,
    numbered_evidence_mode_override: Option<PatchNumberedEvidenceMode>,
}

impl PatchInvocationAdapter {
    pub(crate) fn for_cli_with_anchors(
        execution_anchor_dir: PathBuf,
        display_anchor_dir: Option<PathBuf>,
        patch_source: PatchSourceProvenance,
        numbered_evidence_mode_override: Option<PatchNumberedEvidenceMode>,
    ) -> Self {
        Self {
            transport: TransportInvocation::with_anchors(
                execution_anchor_dir,
                display_anchor_dir,
                patch_source,
            ),
            numbered_evidence_mode_override,
        }
    }

    pub(crate) fn for_workspace(workspace: PathBuf, patch_source: PatchSourceProvenance) -> Self {
        Self {
            transport: TransportInvocation::for_workspace(workspace, patch_source),
            numbered_evidence_mode_override: None,
        }
    }

    pub(crate) fn for_absolute_only(
        anchor_dir: PathBuf,
        patch_source: PatchSourceProvenance,
    ) -> Self {
        Self {
            transport: TransportInvocation::for_execution_only(anchor_dir, patch_source),
            numbered_evidence_mode_override: None,
        }
    }

    pub(crate) fn unanchored(patch_source: PatchSourceProvenance) -> Self {
        Self {
            transport: TransportInvocation::unanchored(patch_source),
            numbered_evidence_mode_override: None,
        }
    }

    pub(crate) fn execute(&self, patch: &str) -> Result<String, String> {
        let outcome = with_patch_mode_override(self.numbered_evidence_mode_override, || {
            apply_patch_program_with_source(
                patch,
                self.transport.execution_anchor_dir(),
                self.transport.source_hint(),
            )
        });
        format_patch_outcome(
            patch,
            &PatchPresentationContext {
                runtime_base_dir: self.transport.execution_anchor_dir().to_path_buf(),
                display_base_dir: self
                    .transport
                    .display_anchor_dir()
                    .map(|path| path.to_path_buf()),
            },
            &outcome,
        )
    }
}

fn patch_mode_env_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

fn with_patch_mode_override<T>(
    mode_override: Option<PatchNumberedEvidenceMode>,
    body: impl FnOnce() -> T,
) -> T {
    let Some(mode_override) = mode_override else {
        return body();
    };
    let _guard = patch_mode_env_guard()
        .lock()
        .expect("patch mode env mutex poisoned");
    let previous = std::env::var_os(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV);
    unsafe {
        std::env::set_var(
            APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV,
            mode_override.as_env_value(),
        );
    }
    let result = body();
    match previous {
        Some(value) => unsafe {
            std::env::set_var(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV, value);
        },
        None => unsafe {
            std::env::remove_var(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV);
        },
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV, PatchInvocationAdapter, PatchNumberedEvidenceMode,
        PatchSourceProvenance, with_patch_mode_override,
    };
    use std::ffi::OsString;

    #[test]
    fn explicit_patch_file_source_is_preserved_in_failure_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("from.txt"), "from\n").expect("seed source");
        std::fs::write(temp.path().join("blocked"), "not a directory").expect("seed blocked file");
        let patch_path = temp.path().join("move-fail.patch");
        let patch = "\
*** Begin Patch
*** Update File: from.txt
*** Move to: blocked/dir/name.txt
@@
-from
+new
*** End Patch
";
        std::fs::write(&patch_path, patch).expect("write patch file");

        let adapter = PatchInvocationAdapter::for_cli_with_anchors(
            temp.path().to_path_buf(),
            Some(temp.path().to_path_buf()),
            PatchSourceProvenance::File(patch_path),
            None,
        );
        let error = adapter.execute(patch).expect_err("patch should fail");

        assert!(error.contains("move-fail.patch:3:1"), "{error}");
        assert!(error.contains("= patch: move-fail.patch"), "{error}");
        assert!(!error.contains(".docutouch/failed-patches/"), "{error}");
    }

    #[test]
    fn custom_cli_anchor_can_display_paths_relative_to_recovered_workspace() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(temp.path().join("src")).expect("create src");
        std::fs::write(temp.path().join("src").join("name.txt"), "from\n").expect("seed source");
        std::fs::write(temp.path().join("blocked"), "not a directory").expect("seed blocked file");
        let patch_path = temp
            .path()
            .join(".docutouch")
            .join("failed-patches")
            .join("retry.patch");
        std::fs::create_dir_all(patch_path.parent().expect("failed-patches dir"))
            .expect("create failed-patches dir");
        let patch = "\
*** Begin Patch
*** Update File: src/name.txt
*** Move to: blocked/dir/name.txt
@@
-from
+new
*** End Patch
";
        std::fs::write(&patch_path, patch).expect("write patch file");

        let adapter = PatchInvocationAdapter::for_cli_with_anchors(
            temp.path().to_path_buf(),
            Some(temp.path().to_path_buf()),
            PatchSourceProvenance::File(patch_path),
            None,
        );
        let error = adapter.execute(patch).expect_err("patch should fail");

        assert!(
            error.contains(".docutouch/failed-patches/retry.patch:3:1"),
            "{error}"
        );
        assert!(
            error.contains("= patch: .docutouch/failed-patches/retry.patch"),
            "{error}"
        );
        assert!(error.contains("TARGET_WRITE_ERROR"), "{error}");
    }

    #[test]
    fn absolute_only_adapter_executes_patch_without_display_anchor() {
        let temp = tempfile::tempdir().expect("tempdir");
        let target = temp.path().join("app.txt");
        std::fs::write(&target, "old\n").expect("seed target");
        let patch = format!(
            "\
*** Begin Patch
*** Update File: {}
@@
-old
+new
*** End Patch
",
            target.display()
        );

        let adapter = PatchInvocationAdapter::for_absolute_only(
            temp.path().to_path_buf(),
            PatchSourceProvenance::Inline,
        );
        let message = adapter.execute(&patch).expect("patch should apply");
        let normalized_target = target.display().to_string().replace('\\', "/");

        assert_eq!(
            std::fs::read_to_string(&target).expect("read target"),
            "new\n"
        );
        assert!(message.contains(&normalized_target), "{message}");
    }

    #[test]
    fn scoped_patch_mode_override_restores_previous_environment() {
        let previous = std::env::var_os(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV);
        unsafe {
            std::env::set_var(
                APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV,
                PatchNumberedEvidenceMode::HeaderOnly.as_env_value(),
            );
        }

        let seen = with_patch_mode_override(Some(PatchNumberedEvidenceMode::Full), || {
            std::env::var_os(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV)
        });

        assert_eq!(seen, Some(OsString::from("full")));
        assert_eq!(
            std::env::var_os(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV),
            Some(OsString::from("header_only"))
        );

        match previous {
            Some(value) => unsafe {
                std::env::set_var(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV, value);
            },
            None => unsafe {
                std::env::remove_var(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV);
            },
        }
    }
}
