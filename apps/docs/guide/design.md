# 设计理念（令牌驱动）

## 三层令牌

```
tokens.css（CSS 变量，明暗两套）
   ↓
Tailwind @theme（把变量暴露为工具类：bg-tint、text-label…）
   ↓
组件（只消费工具类，零硬编码色值）
```

## 核心令牌

| 令牌            | 语义               | 亮色      | 暗色      |
| --------------- | ------------------ | --------- | --------- |
| `--tint`        | 主题色（品牌强调） | `#ff9500` | `#ff9f0a` |
| `--fill`        | 中性填充           | 灰阶      | 灰阶      |
| `--destructive` | 危险色             | 红        | 红        |
| `--separator`   | 分割线             | 黑 α      | 白 α      |
| `--shadow-card` | 卡片投影           | 柔和      | 深色      |

## 动效原则

1. **一律 spring**：`ease-spring` 曲线贯穿 hover / active / 展开 / 弹层
2. **按下即时，回弹过冲**：`active` 状态 0 参与延迟，松手用 spring 过冲模拟物理
3. **只动 composite 属性**：transform / opacity，杜绝布局抖动

## 布局哲学

- flex 优先：`Stack` 是绝大多数布局的答案，`direction` / `spacing` / `align` 三参覆盖 90% 场景
- 间距走 4px 阶梯，圆角走 `rounded-lg / xl / 2xl / full` 四档
