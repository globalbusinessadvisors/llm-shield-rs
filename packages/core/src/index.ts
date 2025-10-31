/**
 * LLM Shield - Enterprise-grade LLM security toolkit
 *
 * @packageDocumentation
 */

export * from './types';
export { LLMShield } from './shield';
export { quickScan } from './quick-scan';
export { version, initialize } from './utils';

// Re-export WASM types for advanced users
export type {
  CacheConfig as WASMCacheConfig,
  ModelTaskWasm,
  ModelVariantWasm,
  CacheStatsWasm,
  MLConfigWasm,
} from './wasm-types';
