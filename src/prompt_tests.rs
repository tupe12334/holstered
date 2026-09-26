use super::*;

#[test]
fn short_prompt_is_kept_whole() {
    assert_eq!(clip("  merge the PR  "), "merge the PR");
}

#[test]
fn request_after_long_paste_survives() {
    let paste = "at frame::poll (src/lib.rs:42)\n".repeat(200);
    let clipped = clip(&format!("{paste}fix this panic"));
    assert!(clipped.ends_with("fix this panic"));
    assert!(clipped.chars().count() <= HEAD + TAIL + 3);
}

#[test]
fn request_before_long_paste_survives() {
    let clipped = clip(&format!("review this diff\n{}", "+ line\n".repeat(500)));
    assert!(clipped.starts_with("review this diff"));
}
