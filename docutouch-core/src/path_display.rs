use std::path::{Path, PathBuf};

pub fn display_path(display_base_dir: Option<&Path>, path: impl AsRef<Path>) -> String {
    let path = normalize_path(path.as_ref());
    if let Some(base_dir) = display_base_dir {
        let base_dir = normalize_path(base_dir);
        if let Ok(relative) = path.strip_prefix(&base_dir)
            && !relative.as_os_str().is_empty()
        {
            return normalize_display_separators(&relative.display().to_string());
        }
    }
    normalize_display_separators(&path.display().to_string())
}

pub fn format_scope(search_paths: &[PathBuf], display_base_dir: Option<&Path>) -> String {
    let display_paths = search_paths
        .iter()
        .map(|path| display_path(display_base_dir, path))
        .collect::<Vec<_>>();
    if display_paths.len() == 1 {
        return display_paths[0].clone();
    }
    format!("[{}]", display_paths.join(", "))
}

fn normalize_path(path: &Path) -> PathBuf {
    PathBuf::from(strip_windows_verbatim_prefix(&path.display().to_string()))
}

fn strip_windows_verbatim_prefix(raw: &str) -> String {
    raw.strip_prefix(r"\\?\").unwrap_or(raw).to_string()
}

fn normalize_display_separators(raw: &str) -> String {
    raw.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_path_relativizes_against_base_dir() {
        let base = Path::new("D:/workspace");
        let path = Path::new("D:/workspace/src/main.rs");
        assert_eq!(display_path(Some(base), path), "src/main.rs");
    }

    #[test]
    fn format_scope_preserves_union_shape() {
        let base = Path::new("D:/workspace");
        let paths = vec![
            PathBuf::from("D:/workspace/src"),
            PathBuf::from("D:/workspace/docs/readme.md"),
        ];
        assert_eq!(format_scope(&paths, Some(base)), "[src, docs/readme.md]");
    }
}
