import { describe, it, expect, beforeEach, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useKeysStore } from '@/stores/keys';
import type { KeyPair } from '@/types/key';

// Mock Tauri APIs
vi.mock('@tauri-apps/plugin-store', () => ({
  LazyStore: vi.fn().mockImplementation(() => ({
    get: vi.fn().mockResolvedValue([]),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
  })),
}));

vi.mock('@tauri-apps/plugin-fs', () => ({
  readDir: vi.fn().mockResolvedValue([]),
  exists: vi.fn().mockResolvedValue(false),
}));

vi.mock('@tauri-apps/api/path', () => ({
  homeDir: vi.fn().mockResolvedValue('/home/test'),
  join: vi.fn().mockImplementation((...args: string[]) => args.join('/')),
  BaseDirectory: {
    Home: 'Home',
  },
}));

describe('🔑 Tests de Gestion des Clés', () => {
  let store: ReturnType<typeof useKeysStore>;

  beforeEach(() => {
    setActivePinia(createPinia());
    store = useKeysStore();
  });

  describe('Test de Génération d\'ID', () => {
    it('addOrUpdateKey génère l\'ID 1 pour la première clé', async () => {
      const newKey = {
        name: 'test-key',
        private: '/path/to/private/key',
        public: '/path/to/public/key.pub',
      };

      const result = await store.addOrUpdateKey(newKey);

      expect(result.id).toBe(1);
      expect(result.name).toBe('test-key');
      expect(result.private).toBe('/path/to/private/key');
      expect(result.public).toBe('/path/to/public/key.pub');
    });

    it('addOrUpdateKey génère le prochain ID disponible', async () => {
      // Ajouter quelques clés d'abord
      await store.addOrUpdateKey({
        name: 'key1',
        private: '/path/key1',
        public: '/path/key1.pub',
      });
      await store.addOrUpdateKey({
        name: 'key3',
        private: '/path/key3',
        public: '/path/key3.pub',
      });
      await store.addOrUpdateKey({
        name: 'key5',
        private: '/path/key5',
        public: '/path/key5.pub',
      });

      // Ajouter une nouvelle clé
      const result = await store.addOrUpdateKey({
        name: 'key6',
        private: '/path/key6',
        public: '/path/key6.pub',
      });

      expect(result.id).toBe(6);
    });
  });

  describe('Test de Recherche de Clé', () => {
    beforeEach(() => {
      // Directly set keys for testing
      (store as any).keys.value = [
        { id: 1, name: 'key1', private: '/path/key1', public: '/path/key1.pub' },
        { id: 2, name: 'key2', private: '/path/key2' },
        { id: 3, name: 'key3', private: '/path/key3', public: '/path/key3.pub' },
      ];
    });

    it('getKey retourne la clé avec l\'ID demandé', () => {
      const result = store.getKey(2);
      expect(result).toEqual({
        id: 2,
        name: 'key2',
        private: '/path/key2',
      });
    });

    it('getKey retourne undefined si la clé n\'existe pas', () => {
      const result = store.getKey(999);
      expect(result).toBeUndefined();
    });

    it('getKey retourne la clé avec tous les champs', () => {
      const result = store.getKey(1);
      expect(result).toEqual({
        id: 1,
        name: 'key1',
        private: '/path/key1',
        public: '/path/key1.pub',
      });
    });
  });
});