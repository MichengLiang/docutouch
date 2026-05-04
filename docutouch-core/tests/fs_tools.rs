use docutouch_core::fs_tools::{
    DirectoryListOptions, ReadFileLineRange, ReadFileOptions, ReadFileSampledViewOptions,
    TimestampField, list_directory, normalize_sampled_view_options,
    parse_read_file_line_range_text, read_file, read_file_with_sampled_view,
};

#[test]
fn list_directory_hides_gitignored_entries_by_default() {
    let temp = tempfile::tempdir().expect("tempdir");
    let repo = temp.path().join("repo");
    std::fs::create_dir_all(repo.join(".git")).expect("git dir");
    std::fs::write(repo.join(".gitignore"), "dist/\ncache.txt\n").expect("gitignore");
    let target = repo.join("app");
    std::fs::create_dir_all(target.join("dist")).expect("dist dir");
    std::fs::write(target.join("dist").join("artifact.txt"), "artifact\n").expect("artifact");
    std::fs::write(target.join("cache.txt"), "cache\n").expect("cache");
    std::fs::write(target.join("visible.txt"), "visible\n").expect("visible");

    let result = list_directory(
        &target,
        DirectoryListOptions {
            max_depth: 3,
            show_hidden: false,
            include_gitignored: false,
            file_types: Vec::new(),
            file_types_not: Vec::new(),
            timestamp_fields: Vec::new(),
        },
    )
    .expect("list directory");

    assert!(!result.tree.contains("dist/"));
    assert!(!result.tree.contains("cache.txt"));
    assert!(result.tree.contains("visible.txt"));
    assert_eq!(result.filtered_gitignored_count, 2);
}

#[test]
fn list_directory_can_include_ripgrep_file_types() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(temp.path().join("src")).expect("src dir");
    std::fs::write(temp.path().join("src").join("main.rs"), "fn main() {}\n").expect("write rust");
    std::fs::write(temp.path().join("src").join("main.cpp"), "int main() {}\n").expect("write cpp");
    std::fs::write(temp.path().join("README.md"), "# notes\n").expect("write markdown");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 3,
            show_hidden: false,
            include_gitignored: true,
            file_types: vec!["rust".to_string()],
            file_types_not: Vec::new(),
            timestamp_fields: Vec::new(),
        },
    )
    .expect("list directory");

    assert!(result.tree.contains("main.rs"));
    assert!(!result.tree.contains("main.cpp"));
    assert!(!result.tree.contains("README.md"));
    assert_eq!(result.filtered_type_count, 2);
    assert!(result.display().contains("2 type"));
}

#[test]
fn list_directory_can_exclude_ripgrep_file_types() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("main.rs"), "fn main() {}\n").expect("write rust");
    std::fs::write(temp.path().join("README.md"), "# notes\n").expect("write markdown");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 2,
            show_hidden: false,
            include_gitignored: true,
            file_types: Vec::new(),
            file_types_not: vec!["markdown".to_string()],
            timestamp_fields: Vec::new(),
        },
    )
    .expect("list directory");

    assert!(result.tree.contains("main.rs"));
    assert!(!result.tree.contains("README.md"));
    assert_eq!(result.filtered_type_count, 1);
}

#[test]
fn list_directory_type_exclusion_wins_over_inclusion() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("main.rs"), "fn main() {}\n").expect("write rust");
    std::fs::write(temp.path().join("README.md"), "# notes\n").expect("write markdown");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 2,
            show_hidden: false,
            include_gitignored: true,
            file_types: vec!["rust".to_string(), "markdown".to_string()],
            file_types_not: vec!["rust".to_string()],
            timestamp_fields: Vec::new(),
        },
    )
    .expect("list directory");

    assert!(!result.tree.contains("main.rs"));
    assert!(result.tree.contains("README.md"));
    assert_eq!(result.filtered_type_count, 1);
}

#[test]
fn list_directory_keeps_max_depth_boundary_dirs_under_type_filter() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(temp.path().join("src").join("nested")).expect("nested dir");
    std::fs::write(
        temp.path().join("src").join("nested").join("main.rs"),
        "fn main() {}\n",
    )
    .expect("write rust");
    std::fs::write(temp.path().join("README.md"), "# notes\n").expect("write markdown");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 1,
            show_hidden: false,
            include_gitignored: true,
            file_types: vec!["rust".to_string()],
            file_types_not: Vec::new(),
            timestamp_fields: Vec::new(),
        },
    )
    .expect("list directory");

    assert!(result.tree.contains("src/"));
    assert!(!result.tree.contains("main.rs"));
    assert!(!result.tree.contains("README.md"));
    assert_eq!(result.filtered_type_count, 1);
}

#[test]
fn list_directory_warns_and_ignores_unknown_ripgrep_file_type() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("README.md"), "# notes\n").expect("write markdown");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 2,
            show_hidden: false,
            include_gitignored: true,
            file_types: vec!["notatype".to_string()],
            file_types_not: Vec::new(),
            timestamp_fields: Vec::new(),
        },
    )
    .expect("unknown type should warn and fall back");

    assert!(result.tree.contains("README.md"));
    assert!(result.display().contains("warnings:"));
    assert!(result.display().contains("notatype"));
    assert!(result.display().contains("type filtering was disabled"));
}

#[test]
fn list_directory_uses_known_ripgrep_file_types_when_some_requested_types_are_unknown() {
    let temp = tempfile::tempdir().expect("tempdir");
    std::fs::write(temp.path().join("main.rs"), "fn main() {}\n").expect("write rust");
    std::fs::write(temp.path().join("README.md"), "# notes\n").expect("write markdown");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 2,
            show_hidden: false,
            include_gitignored: true,
            file_types: vec!["rust".to_string(), "notatype".to_string()],
            file_types_not: Vec::new(),
            timestamp_fields: Vec::new(),
        },
    )
    .expect("known type should remain active");

    assert!(result.tree.contains("main.rs"));
    assert!(!result.tree.contains("README.md"));
    assert!(result.display().contains("notatype"));
}

#[test]
fn read_file_clips_end_of_range_to_eof() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "line 1\nline 2\n").expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some((1, 5).into()),
            show_line_numbers: false,
            max_chars: None,
        },
    )
    .expect("read file");

    assert_eq!(result.content, "line 1\nline 2\n");
}

#[test]
fn read_file_can_render_one_indexed_line_numbers() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "alpha\nbeta\n").expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some((2, 2).into()),
            show_line_numbers: true,
            max_chars: None,
        },
    )
    .expect("read file");

    assert_eq!(result.content, "2 | beta\n");
    assert_eq!(result.start_line, 2);
    assert_eq!(result.line_count, 1);
}

#[test]
fn read_file_aligns_line_numbers_to_widest_visible_line() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    let content = (1..=12)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    std::fs::write(&file_path, content).expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some((9, 12).into()),
            show_line_numbers: true,
            max_chars: None,
        },
    )
    .expect("read file");

    assert_eq!(
        result.content,
        " 9 | line 9\n10 | line 10\n11 | line 11\n12 | line 12\n"
    );
}

#[test]
fn read_file_sampled_view_renders_vertical_omission_markers() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    let content = (1..=8)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    std::fs::write(&file_path, content).expect("write file");

    let result = read_file_with_sampled_view(
        &file_path,
        ReadFileOptions {
            line_range: Some((1, 8).into()),
            show_line_numbers: false,
            max_chars: Some(80),
        },
        Some(ReadFileSampledViewOptions {
            sample_step: 5,
            sample_lines: 2,
        }),
    )
    .expect("read file");

    assert_eq!(result.content, "line 1\nline 2\n...\nline 6\nline 7\n...");
    assert_eq!(result.start_line, 1);
    assert_eq!(result.line_count, 8);
}

#[test]
fn read_file_sampled_view_preserves_line_number_intent() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "abcdefghij\nshort\nklmnopqrstuv\n").expect("write file");

    let result = read_file_with_sampled_view(
        &file_path,
        ReadFileOptions {
            line_range: Some((1, 3).into()),
            show_line_numbers: false,
            max_chars: Some(5),
        },
        Some(ReadFileSampledViewOptions {
            sample_step: 2,
            sample_lines: 1,
        }),
    )
    .expect("read file");

    assert_eq!(
        result.content,
        "abcde...[5 chars omitted]\n...\nklmno...[7 chars omitted]\n"
    );
}

#[test]
fn read_file_sampled_view_can_render_line_numbers_when_requested() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    let content = (1..=12)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    std::fs::write(&file_path, content).expect("write file");

    let result = read_file_with_sampled_view(
        &file_path,
        ReadFileOptions {
            line_range: Some((9, 12).into()),
            show_line_numbers: true,
            max_chars: Some(80),
        },
        Some(ReadFileSampledViewOptions {
            sample_step: 3,
            sample_lines: 2,
        }),
    )
    .expect("read file");

    assert_eq!(
        result.content,
        " 9 | line 9\n10 | line 10\n...\n12 | line 12\n"
    );
}

#[test]
fn read_file_sampled_view_rejects_non_sampled_shapes() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "alpha\nbeta\n").expect("write file");

    let err = read_file_with_sampled_view(
        &file_path,
        ReadFileOptions {
            line_range: Some((1, 2).into()),
            show_line_numbers: false,
            max_chars: Some(80),
        },
        Some(ReadFileSampledViewOptions {
            sample_step: 2,
            sample_lines: 2,
        }),
    )
    .expect_err("sample_lines >= sample_step should fail");

    assert_eq!(
        err.to_string(),
        "sampled view requires 1 <= sample_lines < sample_step"
    );
}

#[test]
fn read_file_sampled_view_rejects_zero_max_chars() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "alpha\nbeta\n").expect("write file");

    let err = read_file_with_sampled_view(
        &file_path,
        ReadFileOptions {
            line_range: Some((1, 2).into()),
            show_line_numbers: false,
            max_chars: Some(0),
        },
        Some(ReadFileSampledViewOptions {
            sample_step: 3,
            sample_lines: 1,
        }),
    )
    .expect_err("zero max_chars should fail");

    assert_eq!(err.to_string(), "read_file requires max_chars >= 1");
}

#[test]
fn read_file_sampled_view_does_not_truncate_when_max_chars_is_omitted() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "abcdefghij\nshort\nklmnopqrstuv\n").expect("write file");

    let result = read_file_with_sampled_view(
        &file_path,
        ReadFileOptions {
            line_range: Some((1, 3).into()),
            show_line_numbers: false,
            max_chars: None,
        },
        Some(ReadFileSampledViewOptions {
            sample_step: 2,
            sample_lines: 1,
        }),
    )
    .expect("read file");

    assert_eq!(result.content, "abcdefghij\n...\nklmnopqrstuv\n");
}

#[test]
fn normalize_sampled_view_options_uses_partial_defaults_without_truncation() {
    let sampled = normalize_sampled_view_options(Some(5), None)
        .expect("normalize should succeed")
        .expect("sampled view should be enabled");

    assert_eq!(sampled.sample_step, 5);
    assert_eq!(sampled.sample_lines, 2);
}

#[test]
fn normalize_sampled_view_options_expands_default_step_for_large_sample_lines() {
    let sampled = normalize_sampled_view_options(None, Some(6))
        .expect("normalize should succeed")
        .expect("sampled view should be enabled");

    assert_eq!(sampled.sample_step, 7);
    assert_eq!(sampled.sample_lines, 6);
}

#[test]
fn normalize_sampled_view_options_requires_sampling_parameters() {
    let sampled = normalize_sampled_view_options(None, None).expect("normalize should succeed");

    assert!(sampled.is_none());
}

#[test]
fn read_file_max_chars_without_sampling_preserves_exact_line_range() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    let content = (1..=6)
        .map(|line| format!("line {line} has more text here\n"))
        .collect::<String>();
    std::fs::write(&file_path, content).expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some((2, 4).into()),
            show_line_numbers: true,
            max_chars: Some(12),
        },
    )
    .expect("read file");

    assert_eq!(
        result.content,
        "2 | line 2 has m...[13 chars omitted]\n3 | line 3 has m...[13 chars omitted]\n4 | line 4 has m...[13 chars omitted]\n"
    );
}

#[test]
fn parse_read_file_line_range_text_supports_slice_like_tail_forms() {
    assert_eq!(
        parse_read_file_line_range_text(":50").expect("parse range"),
        ReadFileLineRange::SliceLike {
            start: None,
            stop: Some(50),
        }
    );
    assert_eq!(
        parse_read_file_line_range_text("-50:-1").expect("parse range"),
        ReadFileLineRange::SliceLike {
            start: Some(-50),
            stop: Some(-1),
        }
    );
}

#[test]
fn parse_read_file_line_range_text_rejects_step_and_zero_endpoints() {
    let step_err = parse_read_file_line_range_text("1:10:2").expect_err("step should fail");
    assert!(step_err.contains("does not support step"));

    let zero_err = parse_read_file_line_range_text(":0").expect_err("zero endpoint should fail");
    assert!(zero_err.contains("must not be 0"));
}

#[test]
fn read_file_slice_like_range_can_read_from_tail_without_total_line_probe() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    let content = (1..=8)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    std::fs::write(&file_path, content).expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some(ReadFileLineRange::SliceLike {
                start: Some(-3),
                stop: None,
            }),
            show_line_numbers: true,
            max_chars: None,
        },
    )
    .expect("read file");

    assert_eq!(result.content, "6 | line 6\n7 | line 7\n8 | line 8\n");
    assert_eq!(result.start_line, 6);
    assert_eq!(result.line_count, 3);
}

#[test]
fn read_file_slice_like_range_can_exclude_the_last_line() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    let content = (1..=4)
        .map(|line| format!("line {line}\n"))
        .collect::<String>();
    std::fs::write(&file_path, content).expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some(ReadFileLineRange::SliceLike {
                start: None,
                stop: Some(-1),
            }),
            show_line_numbers: false,
            max_chars: None,
        },
    )
    .expect("read file");

    assert_eq!(result.content, "line 1\nline 2\nline 3\n");
    assert_eq!(result.start_line, 1);
    assert_eq!(result.line_count, 3);
}

#[test]
fn read_file_slice_like_range_clamps_tail_overshoot_to_file_bounds() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "line 1\nline 2\n").expect("write file");

    let result = read_file(
        &file_path,
        ReadFileOptions {
            line_range: Some(ReadFileLineRange::SliceLike {
                start: Some(-50),
                stop: None,
            }),
            show_line_numbers: false,
            max_chars: None,
        },
    )
    .expect("read file");

    assert_eq!(result.content, "line 1\nline 2\n");
    assert_eq!(result.start_line, 1);
    assert_eq!(result.line_count, 2);
}

#[test]
fn list_directory_can_show_requested_timestamps() {
    let temp = tempfile::tempdir().expect("tempdir");
    let file_path = temp.path().join("notes.md");
    std::fs::write(&file_path, "alpha\n").expect("write file");

    let result = list_directory(
        temp.path(),
        DirectoryListOptions {
            max_depth: 2,
            show_hidden: false,
            include_gitignored: true,
            file_types: Vec::new(),
            file_types_not: Vec::new(),
            timestamp_fields: vec![TimestampField::Modified],
        },
    )
    .expect("list directory");

    assert!(result.tree.contains("modified="));
}
