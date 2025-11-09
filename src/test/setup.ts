import { beforeAll } from 'vitest';

// Mock Tauri APIs
beforeAll(() => {
  // Mock the Tauri store
  global.window = global.window || {};

  // Mock crypto.randomUUID if not available
  if (!global.crypto) {
    global.crypto = {
      randomUUID: () => 'test-uuid-' + Math.random().toString(36).substr(2, 9)
    } as any;
  }
});