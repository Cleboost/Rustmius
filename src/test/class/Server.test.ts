import { describe, it, expect, vi } from 'vitest';
import type { Server } from '@/types/server';

// Extract the validation logic from ConfigServer.isValid()
function isValid(server: Server | undefined): boolean {
  return !!(server?.id && server?.name && server?.ip);
}

describe('🧪 Tests de Validation de Serveur', () => {
  describe('Test de Validation de Configuration', () => {
    it('isValid retourne true pour un serveur valide', () => {
      const validServer: Server = {
        id: 'server-1',
        name: 'Valid Server',
        ip: '192.168.1.1',
        keyID: 1,
      };

      expect(isValid(validServer)).toBe(true);
    });

    it('isValid retourne false si id est manquant', () => {
      const invalidServer = {
        name: 'Server without ID',
        ip: '192.168.1.1',
        keyID: 1,
      } as Server;

      expect(isValid(invalidServer)).toBe(false);
    });

    it('isValid retourne false si name est manquant', () => {
      const invalidServer = {
        id: 'server-1',
        ip: '192.168.1.1',
        keyID: 1,
      } as Server;

      expect(isValid(invalidServer)).toBe(false);
    });

    it('isValid retourne false si ip est manquant', () => {
      const invalidServer = {
        id: 'server-1',
        name: 'Server without IP',
        keyID: 1,
      } as Server;

      expect(isValid(invalidServer)).toBe(false);
    });

    it('isValid retourne false si le serveur est undefined', () => {
      expect(isValid(undefined)).toBe(false);
    });

    it('isValid retourne false si tous les champs sont vides', () => {
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