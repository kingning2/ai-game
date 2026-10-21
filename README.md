# agent-packages

pnpm monorepo。

```
├── apps/          # 应用（待添加）
├── packages/      # 发布包
│   └── shared-ui  # React 组件库
├── eslint.config.js   # ESLint 9 flat config，覆盖全仓库
├── tsconfig.base.json # TS 基础配置，各包 extends
├── .prettierrc.json
└── pnpm-workspace.yaml
```

## 常用命令（根目录执行）

```bash
pnpm install        # 安装全部依赖
pnpm build          # 递归构建所有包
pnpm typecheck      # 递归类型检查
pnpm lint           # ESLint
pnpm format         # Prettier 格式化
```

## 新增一个包

1. `packages/`（或 `apps/`）下新建目录
2. `package.json` 的 `name` 用 `@agent-packages/xxx` 命名
3. `tsconfig.json` 写 `{ "extends": "../../tsconfig.base.json", "include": [...] }`
4. 声明依赖：工具链（typescript/eslint）已在根目录，包内只装自己的运行时/构建依赖
5. 包之间互相引用直接用包名（如 `"@agent-packages/shared-ui": "workspace:*"`）
