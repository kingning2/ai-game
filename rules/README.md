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
- **Rules vs 文档**：`rules/` 告诉 agent **必须怎么做**；组件用法范例见 `apps/web` 演示场与组件源码 TSDoc。两者冲突时以 rules/ 为准并同步修文档。
- **机械强制层**：`.pi/extensions/repo-guard.ts` 把部分条款变成 hook——硬编码色值拦截（ui-design 零硬编码）、按触达文件自动注入对应 rules、build/typecheck 验收提醒。hook 只是执行器，规则内容以本目录为准；改规则改这里，不要在 hook 里复制规则正文。
