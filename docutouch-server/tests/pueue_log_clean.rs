#[path = "../src/pueue_log_clean.rs"]
mod pueue_log_clean;

use pueue_log_clean::clean_task_log_surface;

#[test]
fn keeps_plain_visible_text_unchanged() {
    let input = "alpha\nbeta\n";
    assert_eq!(clean_task_log_surface(input), input);
}

#[test]
fn folds_cr_redraw_history_and_strips_sgr_noise() {
    let input = "\r\x1b[33mLoading...\x1b[0m\r\x1b[32mDone\x1b[0m\n";
    assert_eq!(clean_task_log_surface(input), "Done\n");
}

#[test]
fn preserves_final_cursor_motion_result() {
    let input = "alpha\nbeta\n\x1b[1A\r\x1b[2Krewritten\n";
    assert_eq!(clean_task_log_surface(input), "alpha\nrewritten\n");
}

#[test]
fn drops_osc_title_noise() {
    let input = "\x1b]0;My Terminal Title\x07Hello World\n";
    assert_eq!(clean_task_log_surface(input), "Hello World\n");
}

#[test]
fn rewrites_osc8_hyperlinks_to_label_plus_url() {
    let input = "Docs: \x1b]8;;https://example.com\x07Click here\x1b]8;;\x07\n";
    assert_eq!(
        clean_task_log_surface(input),
        "Docs: Click here (https://example.com)\n"
    );
}

#[test]
fn preserves_final_alt_screen_content_after_exit() {
    let input = "before\n\x1b[?1049hAlt Content\nMore\x1b[?1049l\nafter\n";
    assert_eq!(
        clean_task_log_surface(input),
        "before\nAlt Content\nMore\nafter\n"
    );
}

#[test]
fn clears_bare_cr_redraws_with_wide_characters() {
    let input = "测试测试\rOK\n";
    assert_eq!(clean_task_log_surface(input), "OK\n");
}

#[test]
fn truncated_osc_title_does_not_swallow_following_text() {
    let input = "prefix\x1b]0;title without terminator\nbody\n";
    assert_eq!(clean_task_log_surface(input), "prefix\nbody\n");
}
