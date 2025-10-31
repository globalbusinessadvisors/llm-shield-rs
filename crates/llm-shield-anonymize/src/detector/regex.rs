//! Regex-based Entity Detector

use super::{patterns::*, validators, EntityDetector};
use crate::types::{EntityMatch, EntityType};
use async_trait::async_trait;
use llm_shield_core::Result;

pub struct RegexDetector {}

impl RegexDetector {
    pub fn new() -> Self {
        Self {}
    }

    fn detect_emails(&self, text: &str) -> Vec<EntityMatch> {
        EMAIL_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::Email,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.95,
        }).collect()
    }

    fn detect_phone_numbers(&self, text: &str) -> Vec<EntityMatch> {
        PHONE_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::PhoneNumber,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.90,
        }).collect()
    }

    fn detect_ssn(&self, text: &str) -> Vec<EntityMatch> {
        SSN_PATTERN.find_iter(text).filter_map(|m| {
            if validators::validate_ssn(m.as_str()) {
                Some(EntityMatch {
                    entity_type: EntityType::SSN,
                    value: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.95,
                })
            } else { None }
        }).collect()
    }

    fn detect_credit_cards(&self, text: &str) -> Vec<EntityMatch> {
        CREDIT_CARD_PATTERN.find_iter(text).filter_map(|m| {
            let card_str = m.as_str().replace(['-', ' '], "");
            if validators::validate_luhn(&card_str) {
                Some(EntityMatch {
                    entity_type: EntityType::CreditCard,
                    value: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.95,
                })
            } else { None }
        }).collect()
    }

    fn detect_ip_addresses(&self, text: &str) -> Vec<EntityMatch> {
        IP_ADDRESS_PATTERN.find_iter(text).filter_map(|m| {
            if validators::validate_ipv4(m.as_str()) {
                Some(EntityMatch {
                    entity_type: EntityType::IpAddress,
                    value: m.as_str().to_string(),
                    start: m.start(),
                    end: m.end(),
                    confidence: 0.90,
                })
            } else { None }
        }).collect()
    }

    fn detect_urls(&self, text: &str) -> Vec<EntityMatch> {
        URL_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::Url,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.85,
        }).collect()
    }

    fn detect_dates_of_birth(&self, text: &str) -> Vec<EntityMatch> {
        DATE_OF_BIRTH_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::DateOfBirth,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.75,
        }).collect()
    }

    fn detect_bank_accounts(&self, text: &str) -> Vec<EntityMatch> {
        BANK_ACCOUNT_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::BankAccount,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.70,
        }).collect()
    }

    fn detect_driver_licenses(&self, text: &str) -> Vec<EntityMatch> {
        DRIVER_LICENSE_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::DriverLicense,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.75,
        }).collect()
    }

    fn detect_passports(&self, text: &str) -> Vec<EntityMatch> {
        PASSPORT_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::Passport,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.75,
        }).collect()
    }

    fn detect_medical_records(&self, text: &str) -> Vec<EntityMatch> {
        MEDICAL_RECORD_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::MedicalRecordNumber,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.80,
        }).collect()
    }

    fn detect_postal_codes(&self, text: &str) -> Vec<EntityMatch> {
        POSTAL_CODE_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::PostalCode,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.85,
        }).collect()
    }

    fn detect_person_names(&self, text: &str) -> Vec<EntityMatch> {
        PERSON_NAME_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::Person,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.60,
        }).collect()
    }

    fn detect_addresses(&self, text: &str) -> Vec<EntityMatch> {
        ADDRESS_PATTERN.find_iter(text).map(|m| EntityMatch {
            entity_type: EntityType::Address,
            value: m.as_str().to_string(),
            start: m.start(),
            end: m.end(),
            confidence: 0.65,
        }).collect()
    }

    fn detect_organizations(&self, text: &str) -> Vec<EntityMatch> {
        ORGANIZATION_PATTERN.captures_iter(text).map(|cap| {
            let full = cap.get(0).unwrap();
            let org_name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let suffix = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            EntityMatch {
                entity_type: EntityType::Organization,
                value: format!("{} {}", org_name.trim(), suffix),
                start: full.start(),
                end: full.end(),
                confidence: 0.65,
            }
        }).collect()
    }

    fn remove_overlaps(&self, mut matches: Vec<EntityMatch>) -> Vec<EntityMatch> {
        if matches.is_empty() {
            return matches;
        }

        matches.sort_by_key(|m| m.start);

        let mut result = Vec::new();
        let mut i = 0;

        while i < matches.len() {
            let current = &matches[i];
            let mut best = current.clone();

            let mut j = i + 1;
            while j < matches.len() && matches[j].start < best.end {
                if matches[j].confidence > best.confidence {
                    best = matches[j].clone();
                }
                j += 1;
            }

            result.push(best);
            i = j.max(i + 1);
        }

        result
    }
}

impl Default for RegexDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EntityDetector for RegexDetector {
    async fn detect(&self, text: &str) -> Result<Vec<EntityMatch>> {
        if text.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_matches = Vec::new();

        // Run all detectors
        all_matches.extend(self.detect_emails(text));
        all_matches.extend(self.detect_phone_numbers(text));
        all_matches.extend(self.detect_ssn(text));
        all_matches.extend(self.detect_credit_cards(text));
        all_matches.extend(self.detect_ip_addresses(text));
        all_matches.extend(self.detect_urls(text));
        all_matches.extend(self.detect_dates_of_birth(text));
        all_matches.extend(self.detect_bank_accounts(text));
        all_matches.extend(self.detect_driver_licenses(text));
        all_matches.extend(self.detect_passports(text));
        all_matches.extend(self.detect_medical_records(text));
        all_matches.extend(self.detect_postal_codes(text));
        all_matches.extend(self.detect_person_names(text));
        all_matches.extend(self.detect_addresses(text));
        all_matches.extend(self.detect_organizations(text));

        let filtered = self.remove_overlaps(all_matches);

        Ok(filtered)
    }
}
