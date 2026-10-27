// 构建 axum sidecar（rust/server）并拷贝为 Tauri externalBin 要求的命名：
//   rust/target/<profile>/server.exe
//     → src-tauri/binaries/server-<target-triple>.exe
// 用法：node scripts/build-sidecar.mjs [--release]
//   dev  时默认 debug 构建（快），tauri build 传 --release
import { execSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const desktop = dirname(here); // apps/desktop
const rustWorkspace = join(desktop, '..', '..', 'rust'); // 仓库根 rust/
const release = process.argv.includes('--release');

const profile = release ? 'release' : 'debug';
console.log(`[sidecar] cargo build -p server (${profile}) ...`);
execSync(`cargo build ${release ? '--release' : ''} -p server`, {
  cwd: rustWorkspace,
  stdio: 'inherit',
});

// rustc 1.85+ 支持 --print host-tuple
const targetTriple = execSync('rustc --print host-tuple').toString().trim();
if (!targetTriple) {
  console.error('[sidecar] 无法确定 target triple');
  process.exit(1);
}

const ext = process.platform === 'win32' ? '.exe' : '';
const src = join(rustWorkspace, 'target', profile, `server${ext}`);
const outDir = join(desktop, 'src-tauri', 'binaries');
const dest = join(outDir, `server-${targetTriple}${ext}`);

mkdirSync(outDir, { recursive: true });
copyFileSync(src, dest);
console.log(`[sidecar] ${src} -> ${dest}`);
