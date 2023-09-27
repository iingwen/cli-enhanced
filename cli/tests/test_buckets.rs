use crate::TestCli;
use uuid::Uuid;

#[test]
fn test_bucket_lifecycle() {
    let cli = TestCli::get();
    let owner = TestCli::project();

    let new_bucket_name = format!("{}/test-source-{}", owner, Uuid::new_v4());

    // Create bucket
    let output = cli.run(["create", "bucket", &new_bucket_name]);
    assert!(output.contains(&new_bucket_name), "{}", output);

    let output = cli.