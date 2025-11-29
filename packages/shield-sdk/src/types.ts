/**
 * Severity levels for security issues
 */
export type Severity = 'none' | 'low' | 'medium' | 'high' | 'critical';

/**
 * Categories of security issues
 */
export type Category = 'prompt-injection' | 'secret' | 'pii' | 'toxicity' | 'code-injection' | 'jailbreak';

/**
 * Detected entity in scanned text
 */
export interface Entity {
  /** Type of entity (e.g., 'email', 'ssn', 'api_key') */
  entityType: string;
  /** The matched text */
  text: string;
  /** Start position in the original text */
  start: number;
  /** End position in the original text */
  end: number;
  /** Confidence score (0.0 - 1.0) */
  confidence: number;
}

/**
 * Risk factor describing a security concern
 */
export interface RiskFactor {
  /** Category of the risk */
  category: Category;
  /** Human-readable description */
  description: string;
  /** Severity level */
  severity: Severity;
  /** Confidence score (0.0 - 1.0) */
  confidence: number;
  /** Optional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Result of a security scan
 */
export interface ScanResult {
  /** Whether the input passed all security checks */
  isValid: boolean;
  /** Overall risk score (0.0 - 1.0) */
  riskScore: number;
  /** Sanitized version of the input */
  sanitizedText: string;
  /** List of detected entities */
  entities: Entity[];
  /** List of risk factors */
  riskFactors: RiskFactor[];
  /** Overall severity */
  severity: Severity;
  /** Additional metadata */
  metadata: Record<string, string>;
  /** Scan duration in milliseconds */
  durationMs: number;
}

/**
 * Scanner interface for implementing custom scanners
 */
export interface Scanner {
  /** Unique name of the scanner */
  name: string;
  /** Scan text and return results */
  scan(text: string): Promise<ScanResult>;
}

/**
 * Configuration for a scanner
 */
export interface ScannerConfig {
  /** Whether the scanner is enabled */
  enabled?: boolean;
  /** Custom threshold for this scanner */
  threshold?: number;
  /** Additional scanner-specific options */
  options?: Record<string, unknown>;
}

/**
 * Shield configuration options
 */
export interface ShieldConfig {
  /** Security preset to use */
  preset?: 'strict' | 'standard' | 'permissive';
  /** Risk threshold for short-circuit evaluation (0.0 - 1.0) */
  shortCircuitThreshold?: number;
  /** Whether to run scanners in parallel */
  parallelExecution?: boolean;
  /** Maximum concurrent scanners when parallel execution is enabled */
  maxConcurrent?: number;
  /** Custom input scanners configuration */
  inputScanners?: Record<string, ScannerConfig>;
  /** Custom output scanners configuration */
  outputScanners?: Record<string, ScannerConfig>;
}

/**
 * Options for scan operations
 */
export interface ScanOptions {
  /** Whether to include detailed metadata in results */
  includeMetadata?: boolean;
  /** Maximum text length to scan (truncates if exceeded) */
  maxLength?: number;
  /** Timeout for scan operation in milliseconds */
  timeout?: number;
}
