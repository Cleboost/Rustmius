import { describe, it, expect } from 'vitest';

// Extract the formatBytes function from monitor.vue
function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

describe('📊 Tests de Données', () => {
  describe('Test de Conversion Métriques', () => {
    it('formatBytes convertit correctement 1048576 octets en "1 MB"', () => {
      const result = formatBytes(1048576);
      expect(result).toBe("1 MB");
    });

    it('formatBytes retourne "0 B" pour 0 octets', () => {
      const result = formatBytes(0);
      expect(result).toBe("0 B");
    });

    it('formatBytes convertit correctement les différentes unités', () => {
      expect(formatBytes(512)).toBe("512 B");
      expect(formatBytes(1024)).toBe("1 KB");
      expect(formatBytes(1024 * 1024)).toBe("1 MB");
      expect(formatBytes(1024 * 1024 * 1024)).toBe("1 GB");
      expect(formatBytes(1024 * 1024 * 1024 * 1024)).toBe("1 TB");
    });

    it('formatBytes gère les valeurs décimales', () => {
      expect(formatBytes(1536)).toBe("1.5 KB"); // 1.5 * 1024
      expect(formatBytes(2621440)).toBe("2.5 MB"); // 2.5 * 1024 * 1024
    });
  });
});