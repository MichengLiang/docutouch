use assert_cmd::Command;
use codex_apply_patch::APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV;
use std::fs;
use tempfile::tempdir;

fn apply_patch_command() -> anyhow::Result<Command> {
    Ok(Command::new(assert_cmd::cargo::cargo_bin("apply_patch")))
}

#[test]
fn test_apply_patch_cli_add_and_update() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let file = "cli_test.txt";
    let absolute_path = tmp.path().join(file);

    // 1) Add a file
    let add_patch = format!(
        r#"*** Begin Patch
*** Add File: {file}
+hello
*** End Patch"#
    );
    apply_patch_command()?
        .arg(add_patch)
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(format!("Success. Updated the following files:\nA {file}\n"));
    assert_eq!(fs::read_to_string(&absolute_path)?, "hello\n");

    // 2) Update the file
    let update_patch = format!(
        r#"*** Begin Patch
*** Update File: {file}
@@
-hello
+world
*** End Patch"#
    );
    apply_patch_command()?
        .arg(update_patch)
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(format!("Success. Updated the following files:\nM {file}\n"));
    assert_eq!(fs::read_to_string(&absolute_path)?, "world\n");

    Ok(())
}

#[test]
fn test_apply_patch_cli_stdin_add_and_update() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let file = "cli_test_stdin.txt";
    let absolute_path = tmp.path().join(file);

    // 1) Add a file via stdin
    let add_patch = format!(
        r#"*** Begin Patch
*** Add File: {file}
+hello
*** End Patch"#
    );
    apply_patch_command()?
        .current_dir(tmp.path())
        .write_stdin(add_patch)
        .assert()
        .success()
        .stdout(format!("Success. Updated the following files:\nA {file}\n"));
    assert_eq!(fs::read_to_string(&absolute_path)?, "hello\n");

    // 2) Update the file via stdin
    let update_patch = format!(
        r#"*** Begin Patch
*** Update File: {file}
@@
-hello
+world
*** End Patch"#
    );
    apply_patch_command()?
        .current_dir(tmp.path())
        .write_stdin(update_patch)
        .assert()
        .success()
        .stdout(format!("Success. Updated the following files:\nM {file}\n"));
    assert_eq!(fs::read_to_string(&absolute_path)?, "world\n");

    Ok(())
}

#[test]
fn test_apply_patch_cli_allows_empty_add_file() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("empty.txt");

    apply_patch_command()?
        .arg("*** Begin Patch\n*** Add File: empty.txt\n*** End Patch")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout("Success. Updated the following files:\nA empty.txt\n");

    assert!(absolute_path.exists());
    assert_eq!(fs::read(&absolute_path)?, b"");
    Ok(())
}

#[test]
fn test_apply_patch_cli_warns_when_add_replaces_existing_file() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("notes.md");
    fs::write(&absolute_path, "old\n")?;

    apply_patch_command()?
        .arg("*** Begin Patch\n*** Add File: notes.md\n+new\n*** End Patch")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            "Success. Updated the following files:\nM notes.md\n\nwarning[ADD_REPLACED_EXISTING_FILE]: Add File targeted an existing file and replaced its contents\n  --> notes.md\n  = help: prefer Update File when editing an existing file\n",
        );

    assert_eq!(fs::read_to_string(&absolute_path)?, "new\n");
    Ok(())
}

#[test]
fn test_apply_patch_cli_warns_when_move_replaces_existing_destination() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    fs::write(tmp.path().join("from.txt"), "from\n")?;
    fs::write(tmp.path().join("to.txt"), "dest\n")?;

    apply_patch_command()?
        .arg(
            "*** Begin Patch\n*** Update File: from.txt\n*** Move to: to.txt\n@@\n-from\n+new\n*** End Patch",
        )
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(
            "Success. Updated the following files:\nM to.txt\n\nwarning[MOVE_REPLACED_EXISTING_DESTINATION]: Move to targeted an existing file path and replaced the destination contents\n  --> to.txt\n  = help: prefer a fresh destination path when renaming\n",
        );

    assert_eq!(fs::read_to_string(tmp.path().join("to.txt"))?, "new\n");
    Ok(())
}

#[test]
fn test_apply_patch_cli_preserves_crlf_when_updating_existing_file() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("crlf.txt");
    fs::write(&absolute_path, b"a\r\nb\r\nc\r\n")?;

    apply_patch_command()?
        .arg("*** Begin Patch\n*** Update File: crlf.txt\n@@\n-b\n+x\n+y\n*** End Patch")
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout("Success. Updated the following files:\nM crlf.txt\n");

    assert_eq!(fs::read(&absolute_path)?, b"a\r\nx\r\ny\r\nc\r\n");
    Ok(())
}

#[test]
fn test_apply_patch_cli_preserves_eof_without_trailing_newline() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("no_newline.txt");
    fs::write(&absolute_path, "no newline at end")?;

    apply_patch_command()?
        .arg(
            "*** Begin Patch\n*** Update File: no_newline.txt\n@@\n-no newline at end\n+first line\n+second line\n*** End Patch",
        )
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout("Success. Updated the following files:\nM no_newline.txt\n");

    assert_eq!(fs::read(&absolute_path)?, b"first line\nsecond line");
    Ok(())
}

#[test]
fn test_apply_patch_cli_supports_numbered_context_anchor() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("app.py");
    fs::write(
        &absolute_path,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
    )?;

    apply_patch_command()?
        .arg(
            "*** Begin Patch\n*** Update File: app.py\n@@ 4 | def handler():\n-    value = 1\n+    value = 2\n*** End Patch",
        )
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout("Success. Updated the following files:\nM app.py\n");

    assert_eq!(
        fs::read_to_string(&absolute_path)?,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
    );
    Ok(())
}

#[test]
fn test_apply_patch_cli_supports_dense_numbered_old_side_evidence() -> anyhow::Result<()> {
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("app.py");
    fs::write(
        &absolute_path,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 1\n",
    )?;

    apply_patch_command()?
        .env(APPLY_PATCH_NUMBERED_EVIDENCE_MODE_ENV, "full")
        .arg(
            "*** Begin Patch\n*** Update File: app.py\n@@\n 4 | def handler():\n-5 |     value = 1\n+    value = 2\n*** End Patch",
        )
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout("Success. Updated the following files:\nM app.py\n");

    assert_eq!(
        fs::read_to_string(&absolute_path)?,
        "def handler():\n    value = 0\n\ndef handler():\n    value = 2\n"
    );
    Ok(())
}

#[test]
fn test_apply_patch_cli_header_only_mode_treats_dense_numbered_text_literal() -> anyhow::Result<()>
{
    let tmp = tempdir()?;
    let absolute_path = tmp.path().join("rendered.txt");
    fs::write(&absolute_path, "121 | value = 1\n")?;

    apply_patch_command()?
        .arg(
            "*** Begin Patch\n*** Update File: rendered.txt\n@@\n-121 | value = 1\n+changed\n*** End Patch",
        )
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout("Success. Updated the following files:\nM rendered.txt\n");

    assert_eq!(fs::read_to_string(&absolute_path)?, "changed\n");
    Ok(())
}
