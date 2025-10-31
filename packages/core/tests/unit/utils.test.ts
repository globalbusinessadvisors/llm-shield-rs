import { describe, it, expect } from 'vitest';
import {
  version,
  hashText,
  createScanMetadata,
  formatBytes,
  sleep,
  isBrowser,
  isNode,
  isDeno,
  getRuntime,
} from '../../src/utils';

describe('utils', () => {
  describe('version', () => {
    it('should return version string', () => {
      const v = version();
      expect(typeof v).toBe('string');
      expect(v).toMatch(/^\d+\.\d+\.\d+/);
    });
  });

  describe('hashText', () => {
    it('should create hash from text', () => {
      const hash = hashText('test');
      expect(typeof hash).toBe('string');
      expect(hash.length).toBeGreaterThan(0);
    });

    it('should create consistent hashes', () => {
      const hash1 = hashText('test');
      const hash2 = hashText('test');
      expect(hash1).toBe(hash2);
    });

    it('should create different hashes for different text', () => {
      const hash1 = hashText('test1');
      const hash2 = hashText('test2');
      expect(hash1).not.toBe(hash2);
    });
  });

  describe('createScanMetadata', () => {
    it('should create scan metadata', () => {
      const metadata = createScanMetadata(100, 5, false);

      expect(metadata.durationMs).toBe(100);
      expect(metadata.scannersRun).toBe(5);
      expect(metadata.cached).toBe(false);
      expect(metadata.timestamp).toBeGreaterThan(0);
    });
  });

  describe('formatBytes', () => {
    it('should format 0 bytes', () => {
      expect(formatBytes(0)).toBe('0 Bytes');
    });

    it('should format bytes', () => {
      expect(formatBytes(1024)).toBe('1 KB');
      expect(formatBytes(1024 * 1024)).toBe('1 MB');
      expect(formatBytes(1024 * 1024 * 1024)).toBe('1 GB');
    });

    it('should format with decimals', () => {
      expect(formatBytes(1536)).toBe('1.5 KB');
    });
  });

  describe('sleep', () => {
    it('should sleep for specified time', async () => {
      const start = Date.now();
      await sleep(100);
      const elapsed = Date.now() - start;

      expect(elapsed).toBeGreaterThanOrEqual(90);
      expect(elapsed).toBeLessThan(200);
    });
  });

  describe('runtime detection', () => {
    it('should detect Node.js environment', () => {
      expect(isNode()).toBe(true);
      expect(isBrowser()).toBe(false);
      expect(isDeno()).toBe(false);
    });

    it('should return correct runtime', () => {
      const runtime = getRuntime();
      expect(runtime).toBe('node');
    });
  });
});
