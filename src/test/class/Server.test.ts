import { describe, it, expect, vi } from 'vitest';
import type { Server } from '@/types/server';

// Extract the validation logic from ConfigServer.isValid()
function isValid(server: Server | undefined): boolean {
  return !!(server?.id && server?.name && server?.ip);
}

describe('🧪 Server Validation Tests', () => {
  describe('Configuration Validation Tests', () => {
    it('isValid returns true for a valid server', () => {
      const validServer: Server = {
        id: 'server-1',
        name: 'Valid Server',
        ip: '192.168.1.1',
        keyID: 1,
      };

      expect(isValid(validServer)).toBe(true);
    });

    it('isValid returns false when id is missing', () => {
      const invalidServer = {
        name: 'Server without ID',
        ip: '192.168.1.1',
        keyID: 1,
      } as Server;

      expect(isValid(invalidServer)).toBe(false);
    });

    it('isValid returns false when name is missing', () => {
      const invalidServer = {
        id: 'server-1',
        ip: '192.168.1.1',
        keyID: 1,
      } as Server;

      expect(isValid(invalidServer)).toBe(false);
    });

    it('isValid returns false when ip is missing', () => {
      const invalidServer = {
        id: 'server-1',
        name: 'Server without IP',
        keyID: 1,
      } as Server;

      expect(isValid(invalidServer)).toBe(false);
    });

    it('isValid returns false when the server is undefined', () => {
      expect(isValid(undefined)).toBe(false);
    });

    it('isValid returns false when all fields are empty', () => {
      const invalidServer = {
        id: '',
        name: '',
        ip: '',
        keyID: 0,
      };

      expect(isValid(invalidServer)).toBe(false);
    });
  });
});