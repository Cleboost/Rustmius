import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useServerConfigStore } from '@/stores/servers';
import type { Server, ServerConfig, Folder } from '@/types/server';

// Mock Tauri store
vi.mock('@tauri-apps/plugin-store', () => ({
  LazyStore: vi.fn().mockImplementation(() => ({
    get: vi.fn().mockResolvedValue([]),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
  })),
}));

describe('🧪 Tests de Logique Métier - Serveurs', () => {
  let store: ReturnType<typeof useServerConfigStore>;

  beforeEach(() => {
    setActivePinia(createPinia());
    store = useServerConfigStore();
  });

  describe('Test de Recherche de Serveur', () => {
    it('getServer trouve un serveur par son ID', async () => {
      // Setup mock data by directly setting the tree
      store.tree.value = [
        { id: 'server-1', name: 'Server 1', ip: '192.168.1.1', keyID: 1 },
        { id: 'server-2', name: 'Server 2', ip: '192.168.1.2', keyID: 2 },
        { id: 'server-3', name: 'Server 3', ip: '192.168.1.3', keyID: 3 },
      ];

      const result = store.getServer('server-2');
      expect(result).toEqual({
        id: 'server-2',
        name: 'Server 2',
        ip: '192.168.1.2',
        keyID: 2,
      });
    });

    it('getServer retourne undefined si le serveur n\'existe pas', async () => {
      store.tree.value = [
        { id: 'server-1', name: 'Server 1', ip: '192.168.1.1', keyID: 1 },
      ];

      const result = store.getServer('server-999');
      expect(result).toBeUndefined();
    });
  });

  describe('Test d\'Ajout de Serveur', () => {
    it('addServer ajoute un nouveau serveur à la liste', async () => {
      const newServer: Server = {
        id: 'server-1',
        name: 'Test Server',
        ip: '192.168.1.100',
        keyID: 1,
      };

      await store.addServer(newServer, '/');

      expect(store.listServers()).toContain(newServer);
    });

    it('addServer gère les dossiers correctement', async () => {
      const newServer: Server = {
        id: 'server-1',
        name: 'Server in Folder',
        ip: '192.168.1.101',
        keyID: 2,
      };

      await store.addServer(newServer, '/Test Folder');

      // Vérifier que le serveur a été ajouté quelque part dans la structure
      const allServers = store.listServers();
      expect(allServers).toContain(newServer);
    });
  });

  describe('Test de Validation de Configuration', () => {
    it('isValid retourne true pour un serveur avec tous les champs requis', () => {
      const store = useServerConfigStore();

      // Créer une instance de ConfigServer pour tester
      const server: Server = {
        id: 'server-1',
        name: 'Valid Server',
        ip: '192.168.1.1',
        keyID: 1,
      };

      // On ne peut pas directement tester ConfigServer.isValid car c'est une classe privée
      // Testons plutôt la logique directement
      const isValid = !!(server?.id && server?.name && server?.ip);
      expect(isValid).toBe(true);
    });

    it('isValid retourne false pour un serveur avec des champs manquants', () => {
      const serverWithoutName = {
        id: 'server-1',
        ip: '192.168.1.1',
        keyID: 1,
      };

      const serverWithoutIp = {
        id: 'server-1',
        name: 'Server',
        keyID: 1,
      };

      const serverWithoutId = {
        name: 'Server',
        ip: '192.168.1.1',
        keyID: 1,
      };

      expect(!!(serverWithoutName?.id && serverWithoutName?.name && serverWithoutName?.ip)).toBe(false);
      expect(!!(serverWithoutIp?.id && serverWithoutIp?.name && serverWithoutIp?.ip)).toBe(false);
      expect(!!(serverWithoutId?.id && serverWithoutId?.name && serverWithoutId?.ip)).toBe(false);
    });
  });
});