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
    check_comments_lifecycle(SAMPLE_AUDIO, vec!["--allow-duplicates", "--yes"]);
}

fn check_comments_lifecycle(comments_str: &str, args: Vec<&str>) {
    let annotated_comments: Vec<NewAnnotatedComment> = comments_str
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .unwrap();

    let cli = TestCli::get();
    let source = TestSource::new();

    // Upload our test data
    let output = cli.run_with_stdin(
        ([
            "create",
            "comments",
            &format!("--source={}", source.identifier()),
        ]
        .into_iter()
        .chain(args))
        .collect::<Vec<&str>>(),
        comments_str.as_bytes(),
    );
    assert!(output.is_empty());

    let output = cli.run(["get", "comments", source.identifier()]);
    assert_eq!(output.lines().count(), annotated_comments.len());

    // Assert that all output comments have the same content as the input comments
    let mut output_comments: Vec<Comment> = output
        .lines()
        .map(|line| serde_json::from_str(line).expect("invalid comment"))
        .map(|annotated_comment: AnnotatedComment| annotated_comment.comment)
        .collect();
    output_comments.sort_by(|a, b| a.id.cmp(&b.id));

    let mut input_comments = annotated_comments
        .iter()
        .map(|annotated_comment| annotated_comment.comment.clone())
        .collect::<Vec<NewComment>>();
    input_comments.sort_by(|a, b| a.id.cmp(&b.id));

    for (input_comment, output_comment) in input_comments.iter().zip(output_comments.iter()) {
        assert_eq!(input_comment.id, output_comment.id);
        assert_eq!(input_comment.messages, output_comment.messages);
        assert_e