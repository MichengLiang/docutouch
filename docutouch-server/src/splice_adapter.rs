use crate::transport_shell::{TransportInvocation, TransportSourceProvenance};
use docutouch_core::{SplicePresentationContext, apply_splice_program, format_splice_result};
use std::path::PathBuf;

pub(crate) type SpliceSourceProvenance = TransportSourceProvenance;

#[derive(Clone, Debug)]
pub(crate) struct SpliceInvocationAdapter {
    transport: TransportInvocation,
}

impl SpliceInvocationAdapter {
    pub(crate) fn for_cli(cwd: PathBuf, splice_source: SpliceSourceProvenance) -> Self {
        Self {
            transport: TransportInvocation::for_cli(cwd, splice_source),
        }
    }

    pub(crate) fn for_workspace(workspace: PathBuf) -> Self {
        Self {
            transport: TransportInvocation::for_workspace(
                workspace,
                SpliceSourceProvenance::Inline,
            ),
        }
    }

    pub(crate) fn for_execution_only(
        anchor_dir: PathBuf,
        splice_source: SpliceSourceProvenance,
    ) -> Self {
        Self {
            transport: TransportInvocation::for_execution_only(anchor_dir, splice_source),
        }
    }

    pub(crate) fn unanchored(splice_source: SpliceSourceProvenance) -> Self {
        Self {
            transport: TransportInvocation::unanchored(splice_source),
        }
    }

    pub(crate) fn execute(&self, splice: &str) -> Result<String, String> {
        let context = SplicePresentationContext {
            display_base_dir: self
                .transport
                .display_anchor_dir()
                .map(|path| path.to_path_buf()),
            splice_source: self.transport.source_hint().map(|path| path.to_path_buf()),
        };
        let outcome = apply_splice_program(splice, self.transport.execution_anchor_dir());
        format_splice_result(splice, &context, outcome.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::{SpliceInvocationAdapter, SpliceSourceProvenance};

    #[test]
    fn adapter_executes_splice_against_workspace_anchor() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("seed source");
        let splice = "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | alpha\n*** Append To File: dest.txt\n*** End Splice\n";

        let adapter = SpliceInvocationAdapter::for_workspace(temp.path().to_path_buf());
        let message = adapter.execute(splice).expect("splice should apply");

        assert!(
            message.contains("Success. Updated the following files:"),
            "{message}"
        );
        assert!(message.contains("A dest.txt"), "{message}");
        assert_eq!(
            std::fs::read_to_string(temp.path().join("dest.txt")).expect("read dest"),
            "alpha\n"
        );
    }

    #[test]
    fn adapter_preserves_splice_file_path_in_failure_output() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(temp.path().join("source.txt"), "alpha\n").expect("seed source");
        let splice_path = temp.path().join("broken.splice");
        let splice = "*** Begin Splice\n*** Copy From File: source.txt\n@@\n1 | beta\n*** Append To File: dest.txt\n*** End Splice\n";
        std::fs::write(&splice_path, splice).expect("write splice file");

        let adapter = SpliceInvocationAdapter::for_cli(
            temp.path().to_path_buf(),
            SpliceSourceProvenance::File(splice_path.clone()),
        );
        let error = adapter.execute(splice).expect_err("splice should fail");

        assert!(error.contains("broken.splice:4:1"), "{error}");
        assert!(
            error.contains("error[SPLICE_SOURCE_SELECTION_INVALID]"),
            "{error}"
        );
    }
}
