use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: String,
    pub name: String,
    pub total: u64,
    pub size: u64,
    pub canceled: bool,
}
