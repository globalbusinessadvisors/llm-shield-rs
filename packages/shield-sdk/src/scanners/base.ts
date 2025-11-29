import type { Scanner, ScanResult, Severity, Entity, RiskFactor } from '../types.js';

/**
 * Base scanner class providing common functionality
 */
export abstract class BaseScanner implements Scanner {
  abstract readonly name: string;

  abstract scan(text: string): Promise<ScanResult>;

  /**
   * Create an empty scan result (text is valid)
   */
  protected createValidResult(text: string, durationMs: number): ScanResult {
    return {
      isValid: true,
      riskScore: 0,
      sanitizedText: text,
      entities: [],
      riskFactors: [],
      severity: 'none',
      metadata: {},
      durationMs,
    };
  }

  /**
   * Create a scan result with findings
   */
  protected createResult(
    text: string,
    entities: Entity[],
    riskFactors: RiskFactor[],
    durationMs: number
  ): ScanResult {
    const maxSeverity = this.getMaxSeverity(riskFactors);
    const riskScore = this.calculateRiskScore(riskFactors);

    return {
      isValid: riskFactors.length === 0,
      riskScore,
      sanitizedText: this.sanitize(text, entities),
      entities,
      riskFactors,
      severity: maxSeverity,
      metadata: {},
      durationMs,
    };
  }

  /**
   * Get the maximum severity from risk factors
   */
  protected getMaxSeverity(riskFactors: RiskFactor[]): Severity {
    const severityOrder: Severity[] = ['none', 'low', 'medium', 'high', 'critical'];
    let maxIndex = 0;

    for (const factor of riskFactors) {
      const index = severityOrder.indexOf(factor.severity);
      if (index > maxIndex) {
        maxIndex = index;
      }
    }

    return severityOrder[maxIndex];
  }

  /**
   * Calculate overall risk score from risk factors
   */
  protected calculateRiskScore(riskFactors: RiskFactor[]): number {
    if (riskFactors.length === 0) return 0;

    const severityWeights: Record<Severity, number> = {
      none: 0,
      low: 0.25,
      medium: 0.5,
      high: 0.75,
      critical: 1.0,
    };

    let totalScore = 0;
    for (const factor of riskFactors) {
      totalScore += severityWeights[factor.severity] * factor.confidence;
    }

    return Math.min(1, totalScore / riskFactors.length);
  }

  /**
   * Sanitize text by redacting detected entities
   */
  protected sanitize(text: string, entities: Entity[]): string {
    if (entities.length === 0) return text;

    // Sort entities by start position in reverse order
    const sorted = [...entities].sort((a, b) => b.start - a.start);

    let result = text;
    for (const entity of sorted) {
      const redacted = `[${entity.entityType.toUpperCase()}]`;
      result = result.slice(0, entity.start) + redacted + result.slice(entity.end);
    }

    return result;
  }
}
