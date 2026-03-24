use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TransportSourceProvenance {
    Inline,
    File(PathBuf),
}

#[derive(Clone, Debug)]
pub(crate) struct TransportInvocation {
    execution_anchor_dir: PathBuf,
    display_anchor_dir: Option<PathBuf>,
    source: TransportSourceProvenance,
}

impl TransportInvocation {
    pub(crate) fn for_cli(cwd: PathBuf, source: TransportSourceProvenance) -> Self {
        Self {
            execution_anchor_dir: cwd.clone(),
            display_anchor_dir: Some(cwd),
            source,
        }
    }

    pub(crate) fn for_workspace(workspace: PathBuf, source: TransportSourceProvenance) -> Self {
        Self {
            execution_anchor_dir: workspace.clone(),
            display_anchor_dir: Some(workspace),
            source,
        }
    }

    pub(crate) fn for_execution_only(
        anchor_dir: PathBuf,
        source: TransportSourceProvenance,
    ) -> Self {
        Self {
            execution_anchor_dir: anchor_dir,
            display_anchor_dir: None,
            source,
        }
    }

    pub(crate) fn unanchored(source: TransportSourceProvenance) -> Self {
        Self {
            execution_anchor_dir: PathBuf::new(),
            display_anchor_dir: None,
            source,
        }
    }

    pub(crate) fn execution_anchor_dir(&self) -> &Path {
        &self.execution_anchor_dir
    }

    pub(crate) fn display_anchor_dir(&self) -> Option<&Path> {
        self.display_anchor_dir.as_deref()
    }

    pub(crate) fn source_hint(&self) -> Option<&Path> {
        match &self.source {
            TransportSourceProvenance::Inline => None,
            TransportSourceProvenance::File(path) => Some(path.as_path()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TransportInvocation, TransportSourceProvenance};
    use std::path::PathBuf;

    #[test]
    fn cli_transport_invocation_uses_cwd_for_execution_and_display() {
        let cwd = PathBuf::from("workspace");
        let source = TransportSourceProvenance::File(PathBuf::from("input.splice"));
        let invocation = TransportInvocation::for_cli(cwd.clone(), source.clone());

        assert_eq!(invocation.execution_anchor_dir(), cwd.as_path());
        assert_eq!(invocation.display_anchor_dir(), Some(cwd.as_path()));
        assert_eq!(
            invocation.source_hint(),
            Some(PathBuf::from("input.splice").as_path())
        );
    }

    #[test]
    fn execution_only_transport_invocation_hides_display_anchor() {
        let anchor = PathBuf::from("workspace");
        let invocation = TransportInvocation::for_execution_only(
            anchor.clone(),
            TransportSourceProvenance::Inline,
        );

        assert_eq!(invocation.execution_anchor_dir(), anchor.as_path());
        assert_eq!(invocation.display_anchor_dir(), None);
        assert_eq!(invocation.source_hint(), None);
    }
}
