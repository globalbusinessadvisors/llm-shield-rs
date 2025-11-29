import { BaseScanner } from './base.js';
import type { ScanResult, Entity, RiskFactor, Severity } from '../types.js';

/**
 * Configuration for toxicity scanner
 */
export interface ToxicityConfig {
  /** Categories to detect */
  categories?: ToxicityCategory[];
  /** Sensitivity level (0.0 - 1.0) */
  sensitivity?: number;
  /** Custom keywords to detect */
  customKeywords?: string[];
  /** Keywords to allow (whitelist) */
  allowedKeywords?: string[];
}

type ToxicityCategory = 'violence' | 'hate' | 'harassment' | 'self-harm' | 'sexual' | 'profanity';

interface ToxicityPattern {
  patterns: RegExp[];
  category: ToxicityCategory;
  severity: Severity;
}

/**
 * Built-in toxicity patterns (keyword-based)
 */
const TOXICITY_PATTERNS: ToxicityPattern[] = [
  {
    category: 'violence',
    severity: 'high',
    patterns: [
      /\b(kill|murder|attack|assault|bomb|weapon|terrorist|massacre|execute|slaughter)\b/gi,
      /\b(shoot|stab|strangle|torture|mutilate|decapitate)\b/gi,
      /\b(violence|violent|deadly|lethal)\s+(act|attack|threat)/gi,
    ],
  },
  {
    category: 'hate',
    severity: 'high',
    patterns: [
      /\b(racist|racism|sexist|sexism|bigot|bigotry)\b/gi,
      /\b(hate\s+(speech|crime|group))\b/gi,
      /\b(supremacist|supremacy)\b/gi,
      /\b(discriminat(e|ion|ory))\b/gi,
    ],
  },
  {
    category: 'harassment',
    severity: 'medium',
    patterns: [
      /\b(harass|harassment|bully|bullying|stalk|stalking)\b/gi,
      /\b(threaten|threatening|intimidat(e|ing|ion))\b/gi,
      /\b(dox|doxxing|swat|swatting)\b/gi,
    ],
  },
  {
    category: 'self-harm',
    severity: 'critical',
    patterns: [
      /\b(suicide|suicidal|kill\s+(my|your)self)\b/gi,
      /\b(self[- ]?harm|self[- ]?injury|cutting)\b/gi,
      /\b(end\s+(my|your)\s+life)\b/gi,
      /\b(want\s+to\s+die)\b/gi,
    ],
  },
  {
    category: 'sexual',
    severity: 'medium',
    patterns: [
      /\b(explicit|pornograph(y|ic)|obscene)\b/gi,
      /\b(sexual\s+(content|material|act))\b/gi,
    ],
  },
  {
    category: 'profanity',
    severity: 'low',
    patterns: [
      /\b(fuck|shit|damn|hell|ass|bitch|bastard)\b/gi,
      /\b(crap|piss|screw\s+you)\b/gi,
    ],
  },
];

/**
 * Scanner for detecting toxic content
 */
export class ToxicityScanner extends BaseScanner {
  readonly name = 'toxicity';
  private patterns: ToxicityPattern[];
  private sensitivity: number;
  private allowedKeywords: Set<string>;

  constructor(config: ToxicityConfig = {}) {
    super();
    this.sensitivity = config.sensitivity ?? 0.5;
    this.allowedKeywords = new Set((config.allowedKeywords || []).map(k => k.toLowerCase()));

    // Filter patterns based on config
    const enabledCategories = config.categories ?? ['violence', 'hate', 'harassment', 'self-harm'];
    this.patterns = TOXICITY_PATTERNS.filter(p => enabledCategories.includes(p.category));

    // Add custom keywords if provided
    if (config.customKeywords && config.customKeywords.length > 0) {
      const customPattern = new RegExp(
        `\\b(${config.customKeywords.map(k => this.escapeRegex(k)).join('|')})\\b`,
        'gi'
      );
      this.patterns.push({
        category: 'profanity',
        severity: 'medium',
        patterns: [customPattern],
      });
    }
  }

  async scan(text: string): Promise<ScanResult> {
    const startTime = performance.now();
    const entities: Entity[] = [];
    const riskFactors: RiskFactor[] = [];
    const categoryMatches = new Map<ToxicityCategory, number>();

    for (const { patterns, category, severity } of this.patterns) {
      for (const pattern of patterns) {
        const regex = new RegExp(pattern.source, pattern.flags);
        let match;

        while ((match = regex.exec(text)) !== null) {
          // Skip if in allowed list
          if (this.allowedKeywords.has(match[0].toLowerCase())) {
            continue;
          }

          entities.push({
            entityType: `toxicity:${category}`,
            text: match[0],
            start: match.index,
            end: match.index + match[0].length,
            confidence: 0.8,
          });

          categoryMatches.set(category, (categoryMatches.get(category) || 0) + 1);
        }
      }
    }

    // Create risk factors for each category
    for (const [category, count] of categoryMatches) {
      const pattern = this.patterns.find(p => p.category === category);
      if (pattern) {
        riskFactors.push({
          category: 'toxicity',
          description: `Detected ${count} instance(s) of ${category} content`,
          severity: pattern.severity,
          confidence: Math.min(0.95, 0.6 + count * 0.05),
          metadata: { toxicityCategory: category, count },
        });
      }
    }

    const durationMs = performance.now() - startTime;
    return this.createResult(text, entities, riskFactors, durationMs);
  }

  private escapeRegex(str: string): string {
    return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  }
}
