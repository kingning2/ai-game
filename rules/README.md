# Rules

规范按 ECC 模式分层：**通用层**（与框架无关的项目约定）+ **领域层**（web/UI 具体化）。

```
rules/
├── common/                  # 项目级通用约定（永远适用）
│   ├── project-structure.md # 文件放哪、命名、导出
│   └── coding-style.md      # TS/React 编码规范
└── web/                     # 前端领域（具体覆盖通用）
    ├── ui-design.md         # 令牌 / 用色 / 动效 / 布局
    └── component-guide.md   # 组件使用 + 新增组件流程
```

- **优先级**：`web/` 与 `common/` 冲突时，`web/` 胜（specific overrides general，同 CSS 特异性）。
- **Rules vs 文档**：`rules/` 告诉 agent **必须怎么做**；`apps/docs/guide/` 面向使用者讲**为什么这样设计**。两者冲突时以 rules/ 为准并同步修文档。
