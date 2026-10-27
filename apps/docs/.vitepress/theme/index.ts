import type { Theme } from 'vitepress';
import DefaultTheme from 'vitepress/theme';
import DemoPreview from '../components/DemoPreview.vue';
import './custom.css';
// 文档站 Tailwind 入口（令牌 + 扫描源），demo 里组件真实渲染
import './docs.css';

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('DemoPreview', DemoPreview);
  },
} satisfies Theme;
