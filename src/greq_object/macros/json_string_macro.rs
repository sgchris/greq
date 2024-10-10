
#[macro_export]
macro_rules! json_string {
    ($value:expr) => {
        serde_json::to_string($value).unwrap_or_default()
    };
}