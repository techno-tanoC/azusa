#[derive(Debug, Clone)]
pub struct Item {
    pub url: String,
    pub title: String,
    pub ext: String,
    pub total: u64,
    pub size: u64,
    pub is_canceled: bool,
}
