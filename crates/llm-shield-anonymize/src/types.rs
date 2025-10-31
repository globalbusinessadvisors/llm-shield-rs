//! Core types for anonymization

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Entity types that can be detected and anonymized
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Email,
    CreditCard,
    SSN,
    PhoneNumber,
    IpAddress,
    Url,
    ApiKey,
    AwsAccessKey,
    Location,
    Organization,
    Date,
    MedicalRecordNumber,
    AccountNumber,
    LicensePlate,
    DateOfBirth,
    BankAccount,
    DriverLicense,
    Passport,
    Address,
    PostalCode,
}

impl EntityType {
    /// Convert entity type to placeholder prefix
    pub fn as_str(&self) -> &'static str {
        match self {
            EntityType::Person => "PERSON",
            EntityType::Email => "EMAIL",
            EntityType::CreditCard => "CREDIT_CARD",
            EntityType::SSN => "SSN",
            EntityType::PhoneNumber => "PHONE",
            EntityType::IpAddress => "IP_ADDRESS",
            EntityType::Url => "URL",
            EntityType::ApiKey => "API_KEY",
            EntityType::AwsAccessKey => "AWS_KEY",
            EntityType::Location => "LOCATION",
            EntityType::Organization => "ORGANIZATION",
            EntityType::Date => "DATE",
            EntityType::MedicalRecordNumber => "MRN",
            EntityType::AccountNumber => "ACCOUNT",
            EntityType::LicensePlate => "LICENSE_PLATE",
            EntityType::DateOfBirth => "DATE_OF_BIRTH",
            EntityType::BankAccount => "BANK_ACCOUNT",
            EntityType::DriverLicense => "DRIVER_LICENSE",
            EntityType::Passport => "PASSPORT",
            EntityType::Address => "ADDRESS",
            EntityType::PostalCode => "POSTAL_CODE",
        }
    }
}

/// A detected entity in the input text
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMatch {
    /// Type of entity detected
    pub entity_type: EntityType,
    /// Start byte position in text
    pub start: usize,
    /// End byte position in text
    pub end: usize,
    /// The matched text
    pub value: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Stored mapping between placeholder and original value
#[derive(Debug, Clone, PartialEq)]
pub struct EntityMapping {
    /// Type of entity
    pub entity_type: EntityType,
    /// Original PII value
    pub original_value: String,
    /// Placeholder used in anonymized text
    pub placeholder: String,
    /// Confidence score from detection
    pub confidence: f32,
    /// When this mapping was created
    pub timestamp: SystemTime,
    /// When this mapping expires (for TTL)
    pub expires_at: Option<SystemTime>,
}

/// A placeholder token found in text during deanonymization
#[derive(Debug, Clone, PartialEq)]
pub struct Placeholder {
    /// The full placeholder text (e.g., "[PERSON_1]")
    pub text: String,
    /// Start byte position in text
    pub start: usize,
    /// End byte position in text
    pub end: usize,
}
