/**
 * Type definitions for LLM Shield
 *
 * @module types
 */

/**
 * Scanner type categories
 */
export type ScannerType =
  | 'toxicity'
  | 'prompt-injection'
  | 'secrets'
  | 'pii'
  | 'ban-competitors'
  | 'ban-topics'
  | 'ban-substrings'
  | 'malicious-urls'
  | 'sensitive-output'
  | 'url-reachability';

/**
 * Scanner category
 */
export type ScannerCategory = 'input' | 'output' | 'bidirectional';

/**
 * Severity levels for detections
 */
export type SeverityLevel = 'low' | 'medium' | 'high' | 'critical';

/**
 * Configuration for LLM Shield
 */
export interface LLMShieldConfig {
  /**
   * Cache configuration
   */
  cache?: CacheConfig;

  /**
   * Scanners to enable (empty = all)
   */
  scanners?: ScannerType[];

  /**
   * ML model configuration
   */
  models?: ModelConfig;

  /**
   * Custom risk score thresholds per scanner
   */
  thresholds?: Partial<Record<ScannerType, number>>;

  /**
   * Enable debug logging
   */
  debug?: boolean;

  /**
   * Default timeout for scans in milliseconds
   */
  timeout?: number;
}

/**
 * Cache configuration
 */
export interface CacheConfig {
  /**
   * Maximum number of cached entries
   * @default 1000
   */
  maxSize: number;

  /**
   * Time-to-live in seconds
   * @default 3600
   */
  ttlSeconds: number;

  /**
   * Enable cache persistence (browser only)
   * @default false
   */
  persist?: boolean;
}

/**
 * ML model configuration
 */
export interface ModelConfig {
  /**
   * Enable ML-based scanners
   * @default false
   */
  enabled: boolean;

  /**
   * Model variant to use
   * @default 'INT8'
   */
  variant?: 'FP16' | 'FP32' | 'INT8';

  /**
   * Confidence threshold (0.0 to 1.0)
   * @default 0.5
   */
  threshold?: number;

  /**
   * Fallback to heuristic if ML fails
   * @default true
   */
  fallbackToHeuristic?: boolean;
}

/**
 * Options for scanning operations
 */
export interface ScanOptions {
  /**
   * Scanners to run (overrides config)
   */
  scanners?: ScannerType[];

  /**
   * Skip cache lookup
   * @default false
   */
  skipCache?: boolean;

  /**
   * Timeout in milliseconds
   */
  timeout?: number;

  /**
   * Additional context for the scan
   */
  context?: Record<string, unknown>;

  /**
   * Original prompt (for output scans)
   */
  originalPrompt?: string;
}

/**
 * Result of a security scan
 */
export interface ScanResult {
  /**
   * Whether the text passed all security checks
   */
  isValid: boolean;

  /**
   * Overall risk score (0.0 to 1.0)
   */
  riskScore: number;

  /**
   * List of security issues detected
   */
  detections: Detection[];

  /**
   * Sanitized version of the text (if applicable)
   */
  sanitizedText?: string;

  /**
   * Individual scanner results
   */
  scannerResults: ScannerResult[];

  /**
   * Scan metadata
   */
  metadata: ScanMetadata;
}

/**
 * A detected security issue
 */
export interface Detection {
  /**
   * Scanner that found this issue
   */
  scanner: ScannerType;

  /**
   * Type of issue detected
   */
  type: string;

  /**
   * Severity level
   */
  severity: SeverityLevel;

  /**
   * Risk score contribution (0.0 to 1.0)
   */
  score: number;

  /**
   * Human-readable description
   */
  description: string;

  /**
   * Location in text (if applicable)
   */
  location?: TextLocation;

  /**
   * Additional metadata
   */
  metadata?: Record<string, unknown>;
}

/**
 * Result from an individual scanner
 */
export interface ScannerResult {
  /**
   * Scanner name
   */
  scanner: string;

  /**
   * Whether this scanner passed
   */
  isValid: boolean;

  /**
   * Risk score from this scanner
   */
  riskScore: number;

  /**
   * Risk factors detected
   */
  riskFactors: RiskFactor[];

  /**
   * Entities detected (PII, secrets, etc.)
   */
  entities: Entity[];

  /**
   * Execution time in milliseconds
   */
  executionTimeMs?: number;
}

/**
 * A risk factor contributing to the overall score
 */
export interface RiskFactor {
  /**
   * Type of risk factor
   */
  type: string;

  /**
   * Description
   */
  description: string;

  /**
   * Severity level
   */
  severity: SeverityLevel;

  /**
   * Score contribution (0.0 to 1.0)
   */
  score: number;

  /**
   * Additional metadata
   */
  metadata?: Record<string, unknown>;
}

/**
 * A detected entity (PII, secret, etc.)
 */
export interface Entity {
  /**
   * Type of entity
   */
  entityType: string;

  /**
   * The detected text
   */
  text: string;

  /**
   * Start position in input
   */
  start: number;

  /**
   * End position in input
   */
  end: number;

  /**
   * Confidence score (0.0 to 1.0)
   */
  confidence?: number;
}

/**
 * Text location information
 */
export interface TextLocation {
  /**
   * Start position
   */
  start: number;

  /**
   * End position
   */
  end: number;

  /**
   * Line number (optional)
   */
  line?: number;

  /**
   * Column number (optional)
   */
  column?: number;
}

/**
 * Metadata about a scan operation
 */
export interface ScanMetadata {
  /**
   * Total scan duration in milliseconds
   */
  durationMs: number;

  /**
   * Number of scanners executed
   */
  scannersRun: number;

  /**
   * Whether result was from cache
   */
  cached: boolean;

  /**
   * Timestamp of scan
   */
  timestamp: number;

  /**
   * WASM initialization time (first scan only)
   */
  initTimeMs?: number;
}

/**
 * Information about an available scanner
 */
export interface ScannerInfo {
  /**
   * Scanner name
   */
  name: string;

  /**
   * Scanner category
   */
  type: ScannerCategory;

  /**
   * Scanner version
   */
  version: string;

  /**
   * Description
   */
  description: string;

  /**
   * Whether scanner is currently enabled
   */
  enabled: boolean;

  /**
   * Whether scanner requires ML models
   */
  requiresML?: boolean;
}

/**
 * Result of a batch scan operation
 */
export interface BatchScanResult {
  /**
   * Individual scan results
   */
  results: ScanResult[];

  /**
   * Total processing time
   */
  totalTimeMs: number;

  /**
   * Number of successful scans
   */
  successCount: number;

  /**
   * Number of failed scans
   */
  failureCount: number;

  /**
   * Errors encountered (if any)
   */
  errors?: Array<{
    index: number;
    error: string;
  }>;
}

/**
 * Cache statistics
 */
export interface CacheStats {
  /**
   * Number of cache hits
   */
  hits: number;

  /**
   * Number of cache misses
   */
  misses: number;

  /**
   * Current cache size
   */
  size: number;

  /**
   * Maximum cache size
   */
  maxSize: number;

  /**
   * Hit rate (0.0 to 1.0)
   */
  hitRate: number;
}

/**
 * Quick scan options (for convenience function)
 */
export interface QuickScanOptions extends ScanOptions {
  /**
   * Cache configuration
   */
  cache?: CacheConfig;
}

/**
 * Error types
 */
export class LLMShieldError extends Error {
  constructor(
    message: string,
    public code: string,
    public details?: unknown
  ) {
    super(message);
    this.name = 'LLMShieldError';
  }
}

export class InitializationError extends LLMShieldError {
  constructor(message: string, details?: unknown) {
    super(message, 'INITIALIZATION_ERROR', details);
    this.name = 'InitializationError';
  }
}

export class ScanError extends LLMShieldError {
  constructor(message: string, details?: unknown) {
    super(message, 'SCAN_ERROR', details);
    this.name = 'ScanError';
  }
}

export class TimeoutError extends LLMShieldError {
  constructor(message: string, details?: unknown) {
    super(message, 'TIMEOUT_ERROR', details);
    this.name = 'TimeoutError';
  }
}

export class ValidationError extends LLMShieldError {
  constructor(message: string, details?: unknown) {
    super(message, 'VALIDATION_ERROR', details);
    this.name = 'ValidationError';
  }
}
