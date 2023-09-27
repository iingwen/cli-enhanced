use crate::TestCli;
use uuid::Uuid;

#[test]
fn test_bucket_lifecycle() {
    let cli = TestCli::get();
    let owner = Te