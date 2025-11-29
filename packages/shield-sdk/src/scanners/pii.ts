import { BaseScanner } from './base.js';
import type { ScanResult, Entity, RiskFactor, Severity } from '../types.js';

/**
 * Configuration for PII scanner
 */
export interface PIIConfig {
  /** Types of PII to detect */
  piiTypes?: PIIType[];
  /** Whether to redact detected PII */
  redact?: boolean;
  /** Country-specific formats to detect */
  countries?: string[];
}

type PIIType = 'email' | 'phone' | 'ssn' | 'credit-card' | 'ip-address' | 'address' | 'passport' | 'drivers-license';

interface PIIPattern {
  pattern: RegExp;
  type: string;
  entityType: PIIType;
  severity: Severity;
}

/**
 * Built-in PII patterns
 */
const PII_PATTERNS: PIIPattern[] = [
  // Email
  {
    pattern: /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g,
    type: 'Email Address',
    entityType: 'email',
    severity: 'medium',
  },

  // Phone numbers (international formats)
  {
    pattern: /\b(?:\+?1[-.\s]?)?\(?[2-9]\d{2}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b/g,
    type: 'US Phone Number',
    entityType: 'phone',
    severity: 'medium',
  },
  {
    pattern: /\b\+44\s?[0-9]{4}\s?[0-9]{6}\b/g,
    type: 'UK Phone Number',
    entityType: 'phone',
    severity: 'medium',
  },
  {
    pattern: /\b\+[1-9]\d{1,14}\b/g,
    type: 'International Phone Number',
    entityType: 'phone',
    severity: 'medium',
  },

  // Social Security Numbers
  {
    pattern: /\b\d{3}-\d{2}-\d{4}\b/g,
    type: 'SSN (dashed)',
    entityType: 'ssn',
    severity: 'critical',
  },
  {
    pattern: /\b\d{3}\s\d{2}\s\d{4}\b/g,
    type: 'SSN (spaced)',
    entityType: 'ssn',
    severity: 'critical',
  },
  {
    pattern: /\b\d{9}\b/g,
    type: 'SSN (raw)',
    entityType: 'ssn',
    severity: 'high',
  },

  // Credit Card Numbers
  {
    pattern: /\b4[0-9]{12}(?:[0-9]{3})?\b/g,
    type: 'Credit Card (Visa)',
    entityType: 'credit-card',
    severity: 'critical',
  },
  {
    pattern: /\b5[1-5][0-9]{14}\b/g,
    type: 'Credit Card (Mastercard)',
    entityType: 'credit-card',
    severity: 'critical',
  },
  {
    pattern: /\b3[47][0-9]{13}\b/g,
    type: 'Credit Card (Amex)',
    entityType: 'credit-card',
    severity: 'critical',
  },
  {
    pattern: /\b6(?:011|5[0-9]{2})[0-9]{12}\b/g,
    type: 'Credit Card (Discover)',
    entityType: 'credit-card',
    severity: 'critical',
  },

  // IP Addresses
  {
    pattern: /\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b/g,
    type: 'IPv4 Address',
    entityType: 'ip-address',
    severity: 'low',
  },
  {
    pattern: /\b(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}\b/g,
    type: 'IPv6 Address',
    entityType: 'ip-address',
    severity: 'low',
  },

  // US Passport
  {
    pattern: /\b[A-Z][0-9]{8}\b/g,
    type: 'US Passport Number',
    entityType: 'passport',
    severity: 'high',
  },

  // US Driver's License (general pattern)
  {
    pattern: /\b[A-Z][0-9]{7,8}\b/g,
    type: 'Driver\'s License Number',
    entityType: 'drivers-license',
    severity: 'high',
  },
];

/**
 * Scanner for detecting Personally Identifiable Information (PII)
 */
export class PIIScanner extends BaseScanner {
  readonly name = 'pii';
  private patterns: PIIPattern[];
  private redact: boolean;

  constructor(config: PIIConfig = {}) {
    super();
    this.redact = config.redact ?? true;

    // Filter patterns based on config
    const enabledTypes = config.piiTypes ?? ['email', 'phone', 'ssn', 'credit-card'];
    this.patterns = PII_PATTERNS.filter(p => enabledTypes.includes(p.entityType));
  }

  async scan(text: string): Promise<ScanResult> {
    const startTime = performance.now();
    const entities: Entity[] = [];
    const riskFactors: RiskFactor[] = [];
    const foundTypes = new Map<string, number>();

    for (const { pattern, type, entityType, severity } of this.patterns) {
      const regex = new RegExp(pattern.source, pattern.flags);
      let match;

      while ((match = regex.exec(text)) !== null) {
        // Validate potential matches to reduce false positives
        if (this.validate(match[0], entityType)) {
          entities.push({
            entityType,
            text: this.redact ? this.maskPII(match[0], entityType) : match[0],
            start: match.index,
            end: match.index + match[0].length,
            confidence: this.getConfidence(entityType),
          });

          foundTypes.set(type, (foundTypes.get(type) || 0) + 1);
        }
      }
    }

    // Create risk factors for each type found
    for (const [type, count] of foundTypes) {
      const pattern = this.patterns.find(p => p.type === type);
      if (pattern) {
        riskFactors.push({
          category: 'pii',
          description: `Detected ${count} ${type}(s)`,
          severity: pattern.severity,
          confidence: 0.9,
          metadata: { piiType: type, count },
        });
      }
    }

    const durationMs = performance.now() - startTime;
    return this.createResult(text, entities, riskFactors, durationMs);
  }

  private validate(value: string, type: PIIType): boolean {
    switch (type) {
      case 'credit-card':
        return this.luhnCheck(value.replace(/\D/g, ''));
      case 'ssn':
        // Basic SSN validation (not 000, 666, or 9xx for area)
        const cleaned = value.replace(/\D/g, '');
        if (cleaned.length !== 9) return false;
        const area = parseInt(cleaned.substring(0, 3));
        return area !== 0 && area !== 666 && area < 900;
      case 'email':
        return value.includes('@') && value.includes('.');
      default:
        return true;
    }
  }

  private luhnCheck(num: string): boolean {
    let sum = 0;
    let isEven = false;

    for (let i = num.length - 1; i >= 0; i--) {
      let digit = parseInt(num[i], 10);

      if (isEven) {
        digit *= 2;
        if (digit > 9) {
          digit -= 9;
        }
      }

      sum += digit;
      isEven = !isEven;
    }

    return sum % 10 === 0;
  }

  private getConfidence(type: PIIType): number {
    switch (type) {
      case 'email':
        return 0.95;
      case 'credit-card':
        return 0.99;
      case 'ssn':
        return 0.85;
      case 'phone':
        return 0.75;
      default:
        return 0.7;
    }
  }

  private maskPII(value: string, type: PIIType): string {
    switch (type) {
      case 'email': {
        const [local, domain] = value.split('@');
        return local.substring(0, 2) + '***@' + domain;
      }
      case 'credit-card':
        return '****-****-****-' + value.slice(-4);
      case 'ssn':
        return '***-**-' + value.slice(-4);
      case 'phone':
        return '***-***-' + value.slice(-4);
      default:
        return '****';
    }
  }
}
