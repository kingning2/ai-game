import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['index.ts'],
  format: 'esm',
  dts: true,
  sourcemap: true,
  clean: true,
  // react / react-dom / shared-ui 是 dependencies+peerDependencies，tsdown 默认自动 external
});
