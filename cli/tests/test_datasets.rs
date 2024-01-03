
use backoff::{retry, ExponentialBackoff};
use pretty_assertions::assert_eq;
use reinfer_client::{
    Dataset, EntityDef, EntityName, LabelDef, LabelDefPretrained, LabelDefPretrainedId, LabelGroup,
    LabelGroupName, LabelName, MoonFormFieldDef, Source,
};
use serde_json::json;
use uuid::Uuid;

use crate::{TestCli, TestSource};

pub struct TestDataset {
    full_name: String,
    sep_index: usize,
}

impl TestDataset {
    pub fn new() -> Self {
        let cli = TestCli::get();
        let user = TestCli::project();
        let full_name = format!("{}/test-dataset-{}", user, Uuid::new_v4());
        let sep_index = user.len();

        let output = cli.run(["create", "dataset", &full_name]);
        assert!(output.contains(&full_name));

        Self {
            full_name,
            sep_index,
        }
    }

    pub fn new_args(args: &[&str]) -> Self {
        let cli = TestCli::get();
        let user = TestCli::project();
        let full_name = format!("{}/test-dataset-{}", user, Uuid::new_v4());
        let sep_index = user.len();

        let output = cli.run(["create", "dataset", &full_name].iter().chain(args));
        assert!(output.contains(&full_name));

        Self {
            full_name,
            sep_index,
        }
    }

    pub fn identifier(&self) -> &str {
        &self.full_name
    }

    pub fn owner(&self) -> &str {
        &self.full_name[..self.sep_index]
    }

    pub fn name(&self) -> &str {
        &self.full_name[self.sep_index + 1..]
    }
}

impl Drop for TestDataset {
    fn drop(&mut self) {