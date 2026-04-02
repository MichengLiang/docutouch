use crate::transport_shell::{TransportInvocation, TransportSourceProvenance};
use docutouch_core::{RewritePresentationContext, apply_rewrite_program, format_rewrite_result};
use std::path::PathBuf;

pub(crate) type RewriteSourceProvenance = TransportSourceProvenance;

#[derive(Clone, Debug)]
pub(crate) struct RewriteInvocationAdapter {
    transport: TransportInvocation,
}

impl RewriteInvocationAdapter {
    pub(crate) fn for_cli(cwd: PathBuf, rewrite_source: RewriteSourceProvenance) -> Self {
        Self {
            transport: TransportInvocation::for_cli(cwd, rewrite_source),
        }
    }

    pub(crate) fn for_workspace(workspace: PathBuf) -> Self {
        Self {
            transport: TransportInvocation::for_workspace(
                workspace,
                RewriteSourceProvenance::Inline,
            ),
        }
    }

    pub(crate) fn for_execution_only(
        anchor_dir: PathBuf,
        rewrite_source: RewriteSourceProvenance,
    ) -> Self {
        Self {
            transport: TransportInvocation::for_execution_only(anchor_dir, rewrite_source),
        }
    }

    pub(crate) fn unanchored(rewrite_source: RewriteSourceProvenance) -> Self {
        Self {
            transport: TransportInvocation::unanchored(rewrite_source),
        }
    }

    pub(crate) fn execute(&self, rewrite: &str) -> Result<String, String> {
        let context = RewritePresentationContext {
            display_base_dir: self
                .transport
                .display_anchor_dir()
                .map(|path| path.to_path_buf()),
            rewrite_source: self.transport.source_hint().map(|path| path.to_path_buf()),
        };
        let outcome = apply_rewrite_program(rewrite, self.transport.execution_anchor_dir());
        format_rewrite_result(rewrite, &context, outcome.as_ref())
    }
}
