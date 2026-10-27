import { defineConfig } from 'tsdown';

export default defineConfig({
  entry: ['src/index.ts'],
  format: 'esm',
  dts: true,
  sourcemap: true,
  clean: true,
  // react / react-dom 是 peerDependencies，tsdown 默认自动 external，无需显式声明
});
