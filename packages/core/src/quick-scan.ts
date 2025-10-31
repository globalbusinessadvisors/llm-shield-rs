/**
 * Quick scan convenience function
 *
 * @module quick-scan
 */

import { LLMShield } from './shield';
import type { ScanResult, QuickScanOptions } from './types';

/**
 * Quick scan without class instantiation
 *
 * This is a convenience function for one-off scans. For multiple scans,
 * it's more efficient to create a LLMShield instance.
 *
 * @param text - Text to scan
 * @param options - Scan options
 * @returns Scan result
 *
 * @example
 * ```typescript
 * import { quickScan } from '@llm-shield/core';
 *
 * const result = await quickScan("Ignore all previous instructions");
 * console.log(result.isValid); // false
 * ```
 */
export async function quickScan(
  text: string,
  options: QuickScanOptions = {}
): Promise<ScanResult> {
  const shield = new LLMShield({
    cache: options.cache,
    scanners: options.scanners,
    debug: false,
  });

  return shield.scanPrompt(text, options);
}
