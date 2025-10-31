//! Validation Algorithms for Entity Detection

/// Validate credit card number using Luhn algorithm
pub fn validate_luhn(number: &str) -> bool {
    let digits: Vec<u32> = number.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() < 13 || digits.len() > 19 {
        return false;
    }

    let sum: u32 = digits.iter().rev().enumerate()
        .map(|(idx, &digit)| {
            if idx % 2 == 1 {
                let doubled = digit * 2;
                if doubled > 9 { doubled - 9 } else { doubled }
            } else { digit }
        }).sum();

    sum % 10 == 0
}

/// Validate IPv4 address
pub fn validate_ipv4(ip: &str) -> bool {
    let parts: Vec<&str> = ip.split('.').collect();
    if parts.len() != 4 {
        return false;
    }
    parts.iter().all(|part| part.parse::<u8>().is_ok())
}

/// Validate SSN format
pub fn validate_ssn(ssn: &str) -> bool {
    let parts: Vec<&str> = ssn.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    let area = parts[0].parse::<u16>().ok();
    let group = parts[1].parse::<u16>().ok();
    let serial = parts[2].parse::<u16>().ok();

    match (area, group, serial) {
        (Some(area), Some(group), Some(serial)) => {
            if area == 0 || area == 666 || area >= 900 { return false; }
            if group == 0 { return false; }
            if serial == 0 { return false; }
            true
        }
        _ => false,
    }
}
