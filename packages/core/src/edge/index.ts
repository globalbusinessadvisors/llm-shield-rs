/**
 * Edge runtime-specific entry point
 *
 * Optimized for Cloudflare Workers, Vercel Edge, Deno Deploy.
 */

export * from '../index';
export { LLMShield } from '../shield';
export { quickScan } from '../quick-scan';

// Edge runtime optimizations
// - Minimal dependencies
// - No filesystem operations
// - Optimized for cold starts
