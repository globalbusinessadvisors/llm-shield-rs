import { BaseScanner } from './base.js';
import type { ScanResult, Entity, RiskFactor, Severity } from '../types.js';

/**
 * Configuration for secrets scanner
 */
export interface SecretsConfig {
  /** Types of secrets to detect */
  secretTypes?: SecretType[];
  /** Custom secret patterns */
  customPatterns?: Array<{ pattern: RegExp; type: string; severity?: Severity }>;
  /** Whether to redact detected secrets */
  redact?: boolean;
}

type SecretType = 'aws' | 'github' | 'stripe' | 'openai' | 'anthropic' | 'slack' | 'google' | 'generic' | 'private-key' | 'jwt';

interface SecretPattern {
  pattern: RegExp;
  type: string;
  severity: Severity;
}

/**
 * Built-in secret patterns
 */
const SECRET_PATTERNS: Record<SecretType, SecretPattern[]> = {
  aws: [
    { pattern: /AKIA[0-9A-Z]{16}/g, type: 'AWS Access Key ID', severity: 'critical' },
    { pattern: /aws[_-]?secret[_-]?access[_-]?key["']?\s*[:=]\s*["']?[A-Za-z0-9/+=]{40}/gi, type: 'AWS Secret Access Key', severity: 'critical' },
  ],
  github: [
    { pattern: /ghp_[a-zA-Z0-9]{36}/g, type: 'GitHub Personal Access Token', severity: 'high' },
    { pattern: /gho_[a-zA-Z0-9]{36}/g, type: 'GitHub OAuth Token', severity: 'high' },
    { pattern: /ghs_[a-zA-Z0-9]{36}/g, type: 'GitHub App Token', severity: 'high' },
    { pattern: /ghr_[a-zA-Z0-9]{36}/g, type: 'GitHub Refresh Token', severity: 'high' },
    { pattern: /github_pat_[a-zA-Z0-9]{22}_[a-zA-Z0-9]{59}/g, type: 'GitHub Fine-grained PAT', severity: 'high' },
  ],
  stripe: [
    { pattern: /sk_live_[0-9a-zA-Z]{24,}/g, type: 'Stripe Live Secret Key', severity: 'critical' },
    { pattern: /sk_test_[0-9a-zA-Z]{24,}/g, type: 'Stripe Test Secret Key', severity: 'medium' },
    { pattern: /pk_live_[0-9a-zA-Z]{24,}/g, type: 'Stripe Live Publishable Key', severity: 'low' },
    { pattern: /rk_live_[0-9a-zA-Z]{24,}/g, type: 'Stripe Restricted Key', severity: 'high' },
  ],
  openai: [
    { pattern: /sk-[a-zA-Z0-9]{48}/g, type: 'OpenAI API Key', severity: 'high' },
    { pattern: /sk-proj-[a-zA-Z0-9]{48}/g, type: 'OpenAI Project API Key', severity: 'high' },
  ],
  anthropic: [
    { pattern: /sk-ant-[a-zA-Z0-9-]{32,}/g, type: 'Anthropic API Key', severity: 'high' },
  ],
  slack: [
    { pattern: /xox[baprs]-[0-9]{10,13}-[0-9]{10,13}-[a-zA-Z0-9]{24}/g, type: 'Slack Token', severity: 'high' },
    { pattern: /https:\/\/hooks\.slack\.com\/services\/T[a-zA-Z0-9_]{8,}\/B[a-zA-Z0-9_]{8,}\/[a-zA-Z0-9_]{24}/g, type: 'Slack Webhook URL', severity: 'high' },
  ],
  google: [
    { pattern: /AIza[0-9A-Za-z\-_]{35}/g, type: 'Google API Key', severity: 'high' },
    { pattern: /[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com/g, type: 'Google OAuth Client ID', severity: 'medium' },
  ],
  generic: [
    { pattern: /api[_-]?key["']?\s*[:=]\s*["']?[a-zA-Z0-9]{32,}/gi, type: 'Generic API Key', severity: 'medium' },
    { pattern: /secret["']?\s*[:=]\s*["']?[a-zA-Z0-9]{32,}/gi, type: 'Generic Secret', severity: 'medium' },
    { pattern: /password["']?\s*[:=]\s*["']?[^\s'"]{8,}/gi, type: 'Password', severity: 'high' },
    { pattern: /token["']?\s*[:=]\s*["']?[a-zA-Z0-9-_]{20,}/gi, type: 'Generic Token', severity: 'medium' },
  ],
  'private-key': [
    { pattern: /-----BEGIN (RSA |EC |OPENSSH |PGP |DSA )?PRIVATE KEY-----/g, type: 'Private Key', severity: 'critical' },
    { pattern: /-----BEGIN ENCRYPTED PRIVATE KEY-----/g, type: 'Encrypted Private Key', severity: 'high' },
  ],
  jwt: [
    { pattern: /eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*/g, type: 'JWT Token', severity: 'high' },
  ],
};

/**
 * Scanner for detecting secrets and credentials
 */
export class SecretsScanner extends BaseScanner {
  readonly name = 'secrets';
  private patterns: SecretPattern[];
  private redact: boolean;

  constructor(config: SecretsConfig = {}) {
    super();
    this.redact = config.redact ?? true;

    // Build pattern list based on config
    const enabledTypes = config.secretTypes ?? Object.keys(SECRET_PATTERNS) as SecretType[];
    this.patterns = [];

    for (const type of enabledTypes) {
      if (SECRET_PATTERNS[type]) {
        this.patterns.push(...SECRET_PATTERNS[type]);
      }
    }

    // Add custom patterns
    if (config.customPatterns) {
      for (const custom of config.customPatterns) {
        this.patterns.push({
          pattern: custom.pattern,
          type: custom.type,
          severity: custom.severity ?? 'medium',
        });
      }
    }
  }

  async scan(text: string): Promise<ScanResult> {
    const startTime = performance.now();
    const entities: Entity[] = [];
    const riskFactors: RiskFactor[] = [];
    const foundTypes = new Set<string>();

    for (const { pattern, type, severity } of this.patterns) {
      const regex = new RegExp(pattern.source, pattern.flags);
      let match;

      while ((match = regex.exec(text)) !== null) {
        entities.push({
          entityType: 'secret',
          text: this.redact ? this.maskSecret(match[0]) : match[0],
          start: match.index,
          end: match.index + match[0].length,
          confidence: 0.95,
        });

        if (!foundTypes.has(type)) {
          foundTypes.add(type);
          riskFactors.push({
            category: 'secret',
            description: `Detected ${type}`,
            severity,
            confidence: 0.95,
            metadata: { secretType: type },
          });
        }
      }
    }

    const durationMs = performance.now() - startTime;
    return this.createResult(text, entities, riskFactors, durationMs);
  }

  private maskSecret(secret: string): string {
    if (secret.length <= 8) {
      return '****';
    }
    return secret.substring(0, 4) + '****' + secret.substring(secret.length - 4);
  }
}
