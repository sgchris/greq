pub trait FromString {
    fn from_string(contents: &str) -> Result<Self, String>
    where Self: Sized;
}