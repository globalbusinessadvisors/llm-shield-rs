//! Regex Patterns for Entity Detection

use regex::Regex;
use std::sync::LazyLock;

pub static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

pub static PHONE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(\+\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}").unwrap()
});

pub static CREDIT_CARD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b|\b\d{15}\b").unwrap()
});

pub static SSN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap()
});

pub static IP_ADDRESS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap()
});

pub static URL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://[^\s<>"]+|www\.[^\s<>"]+"#).unwrap()
});

pub static DATE_OF_BIRTH_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b(?:\d{1,2}[-/]\d{1,2}[-/]\d{4}|\d{4}[-/]\d{1,2}[-/]\d{1,2})\b").unwrap()
});

pub static BANK_ACCOUNT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{8,17}\b").unwrap()
});

pub static DRIVER_LICENSE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Z]\d{7}\b|\b\d{8,12}\b").unwrap()
});

pub static PASSPORT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Z]{1,2}\d{7,8}\b").unwrap()
});

pub static MEDICAL_RECORD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\bMRN[-:]?\d{6,10}\b|\b\d{6,10}\b").unwrap()
});

pub static POSTAL_CODE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{5}(?:-\d{4})?\b").unwrap()
});

pub static PERSON_NAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Z][a-z]+\s+[A-Z][a-z]+\b").unwrap()
});

pub static ADDRESS_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d+\s+[A-Z][a-z]+\s+(?:Street|St|Avenue|Ave|Road|Rd|Drive|Dr|Lane|Ln|Boulevard|Blvd)\b").unwrap()
});

pub static ORGANIZATION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b([A-Z][A-Za-z\s]+?)\s+(Inc|Corp|LLC|Ltd|Company|Co)\b").unwrap()
});
