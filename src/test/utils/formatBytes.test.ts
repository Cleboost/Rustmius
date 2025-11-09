import { describe, it, expect } from 'vitest';

// Extract the formatBytes function from monitor.vue
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

describe('📊 Data Tests', () => {
  describe('Metric Conversion Tests', () => {
    it('formatBytes correctly converts 1048576 bytes to "1 MB"', () => {
      const result = formatBytes(1048576);
      expect(result).toBe("1 MB");
    });

    it('formatBytes returns "0 B" for 0 bytes', () => {
      const result = formatBytes(0);
      expect(result).toBe("0 B");
    });

    it('formatBytes correctly converts across units', () => {
      expect(formatBytes(512)).toBe("512 B");
      expect(formatBytes(1024)).toBe("1 KB");
      expect(formatBytes(1024 * 1024)).toBe("1 MB");
      expect(formatBytes(1024 * 1024 * 1024)).toBe("1 GB");
      expect(formatBytes(1024 * 1024 * 1024 * 1024)).toBe("1 TB");
    });

    it('formatBytes handles decimal values', () => {
      expect(formatBytes(1536)).toBe("1.5 KB"); // 1.5 * 1024
      expect(formatBytes(2621440)).toBe("2.5 MB"); // 2.5 * 1024 * 1024
    });
  });
});