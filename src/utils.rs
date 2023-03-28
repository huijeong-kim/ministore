pub fn humansize_to_integer(size_str: &String) -> Result<u64, String> {
    let size_int = size_str
        .trim_end_matches(char::is_alphabetic)
        .parse::<u64>()
        .map_err(|e| e.to_string())?;
    let multiplier = match size_str.chars().last().unwrap_or_default() {
        'k' | 'K' => 1024,
        'm' | 'M' => 1024 * 1024,
        'g' | 'G' => 1024 * 1024 * 1024,
        _ => 1,
    };

    Ok(size_int * multiplier)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converting_kmg_should_success() {
        assert_eq!(humansize_to_integer(&"20k".to_string()).unwrap(), 20 * 1024);
        assert_eq!(humansize_to_integer(&"20K".to_string()).unwrap(), 20 * 1024);

        assert_eq!(humansize_to_integer(&"10m".to_string()).unwrap(), 10 * 1024 * 1024);
        assert_eq!(humansize_to_integer(&"10M".to_string()).unwrap(), 10 * 1024 * 1024);

        assert_eq!(humansize_to_integer(&"6g".to_string()).unwrap(), 6 * 1024 * 1024 * 1024);
        assert_eq!(humansize_to_integer(&"6G".to_string()).unwrap(), 6 * 1024 * 1024 * 1024);

        assert_eq!(humansize_to_integer(&"100000".to_string()).unwrap(), 100000);
    }
}