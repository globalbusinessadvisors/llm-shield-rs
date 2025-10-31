/**
 * Browser-specific entry point
 *
 * This module provides browser-optimized exports with smaller bundle size.
 */

export * from '../index';
export { LLMShield } from '../shield';
export { quickScan } from '../quick-scan';

// Browser-specific initialization
if (typeof window !== 'undefined') {
  // Auto-initialize on page load if needed
  // Can be configured via window.LLMShieldConfig
}
