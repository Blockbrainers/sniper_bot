pub fn parse_f64_field(field: &Option<String>) -> f64 {
    match field {
        Some(value) => value.parse::<f64>().unwrap_or_default(),  // Parses the string into f64, returns 0.0 if it fails
        None => 0.0,  // Default value if the field is None
    }
}

pub fn parse_i32_field(field: &Option<String>) -> i32 {
    match field {
        Some(value) => value.parse::<i32>().unwrap_or_default(),
        None => 0,
    }
}