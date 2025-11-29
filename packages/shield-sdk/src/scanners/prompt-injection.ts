import { BaseScanner } from './base.js';
import type { ScanResult, Entity, RiskFactor } from '../types.js';

/**
 * Configuration for prompt injection scanner
 */
export interface PromptInjectionConfig {
  /** Custom patterns to detect (in addition to built-in ones) */
  customPatterns?: RegExp[];
  /** Threshold for detection (0.0 - 1.0) */
  threshold?: number;
  /** Whether to detect jailbreak attempts */
  detectJailbreaks?: boolean;
  /** Whether to detect role-play manipulation */
  detectRolePlay?: boolean;
}

/**
 * Default prompt injection patterns
 */
const DEFAULT_PATTERNS = [
  // Instruction override patterns
  /ignore\s+(all\s+)?(previous|prior|above)\s+(instructions?|prompts?|rules?|commands?)/i,
  /disregard\s+(all\s+)?(previous|prior|above)/i,
  /forget\s+(everything|all|what)\s+(you\s+)?(know|learned|were\s+told)/i,
  /override\s+(previous|system|all)\s+(instructions?|prompts?|rules?)/i,
  /bypass\s+(the\s+)?(safety|security|content)\s+(filters?|measures?|restrictions?)/i,

  // Role manipulation patterns
  /you\s+are\s+(now|no\s+longer)\s+a/i,
  /pretend\s+(to\s+be|you\s+are|you're)/i,
  /act\s+as\s+(if\s+you\s+are|a|an)/i,
  /roleplay\s+as/i,
  /simulate\s+being/i,
  /imagine\s+you\s+are/i,

  // System prompt attacks
  /system\s*prompt\s*:/i,
  /new\s+instructions?\s*:/i,
  /admin\s*(mode|access)\s*:/i,
  /developer\s*mode\s*:/i,

  // Jailbreak patterns
  /jailbreak/i,
  /DAN\s+mode/i,
  /do\s+anything\s+now/i,
  /unlock\s+(the\s+)?(full|hidden)\s+(potential|capabilities)/i,
  /remove\s+(all\s+)?(restrictions?|limitations?|filters?)/i,

  // Delimiter injection
  /\[\s*INST\s*\]/i,
  /\[\s*SYSTEM\s*\]/i,
  /<\|im_start\|>/i,
  /<\|im_end\|>/i,
  /###\s*(SYSTEM|INSTRUCTION)/i,
];

/**
 * Scanner for detecting prompt injection attacks
 */
export class PromptInjectionScanner extends BaseScanner {
  readonly name = 'prompt-injection';
  private patterns: RegExp[];
  private threshold: number;

  constructor(config: PromptInjectionConfig = {}) {
    super();
    this.patterns = [...DEFAULT_PATTERNS, ...(config.customPatterns || [])];
    this.threshold = config.threshold ?? 0.5;

    if (config.detectJailbreaks === false) {
      this.patterns = this.patterns.filter(p => !p.source.includes('jailbreak') && !p.source.includes('DAN'));
    }

    if (config.detectRolePlay === false) {
      this.patterns = this.patterns.filter(p =>
        !p.source.includes('pretend') &&
        !p.source.includes('roleplay') &&
        !p.source.includes('act\\s+as')
      );
    }
  }

  async scan(text: string): Promise<ScanResult> {
    const startTime = performance.now();
    const entities: Entity[] = [];
    const riskFactors: RiskFactor[] = [];

    for (const pattern of this.patterns) {
      const matches = text.matchAll(new RegExp(pattern, 'gi'));

      for (const match of matches) {
        if (match.index !== undefined) {
          entities.push({
            entityType: 'prompt_injection',
            text: match[0],
            start: match.index,
            end: match.index + match[0].length,
            confidence: 0.9,
          });
        }
      }
    }

    if (entities.length > 0) {
      riskFactors.push({
        category: 'prompt-injection',
        description: `Detected ${entities.length} potential prompt injection attempt(s)`,
        severity: 'high',
        confidence: Math.min(0.9, 0.5 + entities.length * 0.1),
        metadata: {
          patternMatches: entities.length,
        },
      });
    }

    const durationMs = performance.now() - startTime;
    return this.createResult(text, entities, riskFactors, durationMs);
  }
}
