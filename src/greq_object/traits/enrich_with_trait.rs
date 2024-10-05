pub trait EnrichWith {
    // merge with another object, but values that already set
    fn enrich_with(&mut self, object_to_merge: &Self) -> Result<(), String>
    where Self: Sized;
}