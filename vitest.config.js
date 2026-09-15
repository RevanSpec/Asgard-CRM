import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/domain/**/*.test.js'],
    setupFiles: ['./src/domain/__tests__/setup.js'],
  },
});
