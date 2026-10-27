# 文件放置规范

> 原则：**一类文件只有一个家**。放错位置的文件等于不存在的文件。

## Monorepo 布局

```
agent-packages/
├── AGENTS.md                  # 规范入口（agent 必读）
├── rules/                     # 工程规范（分层见 rules/README.md）
├── packages/
│   ├── shared/                # 纯逻辑：类型、请求，禁止 import react
│   └── shared-ui/             # 组件库本体
│       ├── src/
│       │   ├── <Name>/index.tsx   # 一个组件一个文件夹（PascalCase）
│       │   ├── layout/            # 布局组件特例：Stack/Divider/Container/NavBar/SectionHeader
│       │   ├── cn.ts              # 类名工具
│       │   └── index.ts           # 唯一 barrel：所有公开导出
│       └── styles/
│           ├── index.css          # 样式唯一入口（tailwind + tokens + reset）
│           ├── tokens.css         # 设计令牌（明暗两套），唯一允许出现色值的地方
│           ├── animations.css     # 预设动画
│           └── components.css     # 组件级全局样式（少用）
├── apps/
│   ├── web/                   # 演示场：src/App.tsx 按 Section 编排全部组件
│   ├── docs/                  # VitePress 文档站
│   │   ├── components/<kebab-case>.md   # 每组件一页
│   │   ├── demos/<Name>/basic.tsx       # 可交互 demo（PascalCase 文件夹）
│   │   ├── demos/index.ts               # demo 注册表
│   │   ├── guide/                       # 设计理念、快速上手
│   │   └── .vitepress/config.mts        # 侧边栏（新页必登记）
│   └── desktop/               # Tauri
└── rust/                      # Rust 服务端
```

## 放置判定表

| 你要新建的东西 | 放这里                                                       | 命名                        |
| -------------- | ------------------------------------------------------------ | --------------------------- |
| UI 组件        | `packages/shared-ui/src/<Name>/index.tsx`                    | 文件夹 PascalCase，导出同名 |
| 组件用到的类型 | 同组件文件内导出（小类型）或 `src/<Name>/types.ts`（大类型） | PascalCase                  |
| 纯工具函数     | `packages/shared/src/`                                       | camelCase                   |
| 设计令牌/动画  | `packages/shared-ui/styles/`                                 | kebab-case class            |
| 演示场示例     | `apps/web/src/App.tsx` 新 `<Section>`                        | index 递增                  |
| 文档页         | `apps/docs/components/<kebab-case>.md`                       | 与组件名对应                |
| 文档 demo      | `apps/docs/demos/<Name>/basic.tsx`                           | 与组件文件夹同名            |
| 布局类组件     | `packages/shared-ui/src/layout/`                             | PascalCase                  |

## 新组件交付 checklist（缺一不可）

1. `packages/shared-ui/src/<Name>/index.tsx` — 组件 + 类型
2. `packages/shared-ui/src/index.ts` — barrel 追加导出（按 布局/基础/展示/…/AI 分组注释放对区块）
3. `apps/docs/demos/<Name>/basic.tsx` — demo + `demos/index.ts` 注册
4. `apps/docs/components/<name>.md` — 文档页（描述 / 何时使用 / `<DemoPreview name="<Name>Basic" />` / API 表）
5. `apps/docs/.vitepress/config.mts` — 侧边栏登记（放对分组）
6. `apps/web/src/App.tsx` — 新增 Section（可选，交互类组件必须）
7. `README.md` 组件清单登记（可选，重要组件）

## 禁止事项

- 禁止在 `apps/*` 里写新组件（apps 只消费 `@agent-packages/shared-ui`）
- 禁止绕过 barrel 直接深路径导入（`@agent-packages/shared-ui` 根导入是唯一方式；样式经 `./styles.css`）
- 禁止在 `packages/shared` 里 import react / DOM
- 禁止在组件文件里写 `*.css`（样式统一走 styles/ + Tailwind 工具类）
