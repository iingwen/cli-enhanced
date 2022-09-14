use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct Id(pub String);

#[derive(Debug, Clone, Deserialize, Se