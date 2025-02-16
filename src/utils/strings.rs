// Define a trait for the extension method
pub trait RemoveBom {
    fn remove_bom(&self) -> Self;
}

// Implement the trait for the String type
impl RemoveBom for String {
    fn remove_bom(&self) -> Self {
        if self.starts_with('\u{feff}') {
            self.trim_start_matches('\u{feff}').to_string()
        } else {
            self.clone()
        }
    }
}
