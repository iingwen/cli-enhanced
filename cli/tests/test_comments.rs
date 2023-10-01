use crate::{TestCli, TestDataset, TestSource};
use anyhow::anyhow;
use backoff::{retry, ExponentialBackoff};
use chrono::DateTime;
use pretty_assertions::assert_eq;
use reinfer_client::{AnnotatedComment, Comment, NewAnnotatedComment, NewComment};

#[test]
fn test_comments_lifecycle_basic() {
    const SAMPLE_BASIC: &str = include_str!("./samples/basic.jsonl");
    check_comments_lifecycle(SAMPLE_BASIC, vec!["--allow-duplicates", "--yes"]);
}

#[test]
fn test_comments_lifecycle_labellings() {
    const SAMPLE_LABELLING: &str = include_str!("./samples/labelling.jsonl");
    check_comments_lifecycle(SAMPLE_LABELLING, vec!["--allow-duplicates", "--yes"]);
}

#[test]
fn test_comments_lifecycle_legacy_labelling() {
    const SAMPLE_LEGACY_LABELLING: &str = include_str!("./samples/legacy_labelling.jsonl");
    check_comments_lifecycle(SAMPLE_LEGACY_LABELLING, vec!["--allow-duplicates", "--yes"]);
}

#[test]
fn test_comments_lifecycle_moon_forms() {
    const SAMPLE_MOON_LABELLING: &str = include_str!("./samples/moon_forms.jsonl");
    // check without moon forms
    check_comments_lifecycle(SAMPLE_MOON_LABELLING, vec!["--allow-duplicates", "--yes"]);
    // and with moon forms
    check_comments_lifecycle(
        SAMPLE_MOON_LABELLING,
        vec!["--allow-duplicates", "--yes", "--use-moon-forms"],
    );
}

#[test]
fn test_comments_lifecycle_audio() {
    const SAMPLE_AUDIO: &str = include_str!("./samples/audio.jsonl");
    check_com