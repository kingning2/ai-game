import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath, URL } from 'node:url';
import { defineConfig } from 'vitepress';

// 参考组件库文档站结构：nav + 分组侧边栏 + 每组件一页
export default defineConfig({
  title: 'Shared UI',
  description: 'Apple HIG 风格 · 令牌驱动的 React 组件库',
  lang: 'zh-CN',
  head: [['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }]],

  vite: {
    // React Compiler：oxc 原生 Rust 实现（oxc-transform-react），零 Babel
    // exclude：插件默认 include 匹配 .ts/.tsx（含 query），会把 VitePress 的
    // Vue SFC 虚拟模块（*.vue?vue&type=script&lang.ts）误当 React 文件，需排除
    plugins: [react({ compiler: true, exclude: [/\/node_modules\//, /\.vue\?/] }), tailwindcss()],
    esbuild: { jsx: 'automatic' },
    resolve: {
      alias: [
        // 顺序敏感：子路径先映射，再映射包根
        {
          find: '@agent-packages/shared-ui/styles.css',
          replacement: fileURLToPath(
            new URL('../../../packages/shared-ui/styles/index.css', import.meta.url),
          ),
        },
        {
          find: /^@agent-packages\/shared-ui$/,
          replacement: fileURLToPath(
            new URL('../../../packages/shared-ui/src/index.ts', import.meta.url),
          ),
        },
      ],
    },
  },

  themeConfig: {
    search: {
      provider: 'local',
      options: {
        translations: {
          button: { buttonText: '搜索文档', buttonAriaLabel: '搜索文档' },
          modal: {
            noResultsText: '没有找到结果',
            resetButtonTitle: '清除查询条件',
            footer: { selectText: '选择', navigateText: '切换', closeText: '关闭' },
          },
        },
      },
    },
    outline: { level: [2, 3], label: '本页目录' },
    docFooter: { prev: '上一页', next: '下一页' },
    lastUpdated: { text: '最后更新于', formatOptions: { dateStyle: 'short' } },
    returnToTopLabel: '回到顶部',
    sidebarMenuLabel: '菜单',
    socialLinks: [{ icon: 'github', link: 'https://github.com/your-org/agent-packages' }],

    nav: [
      { text: '指南', link: '/guide/getting-started', activeMatch: '/guide/' },
      { text: '组件', link: '/components/button', activeMatch: '/components/' },
    ],

    sidebar: {
      '/guide/': [
        {
          text: '开始',
          items: [
            { text: '快速上手', link: '/guide/getting-started' },
            { text: '设计理念（令牌驱动）', link: '/guide/design' },
          ],
        },
      ],
      '/components/': [
        // 分层与根 README 一致：基础 = 零依赖原子；高级 = headless/Portal；AI = Agent 场景件（单智能体/多智能体两组）
        {
          text: '基础 · 布局',
          items: [
            { text: 'Stack 层叠容器', link: '/components/stack' },
            { text: 'Divider 分割线', link: '/components/divider' },
            { text: 'Container 容器', link: '/components/container' },
            { text: 'NavBar 导航栏', link: '/components/nav-bar' },
            { text: 'SectionHeader 区块标题', link: '/components/section-header' },
          ],
        },
        {
          text: '基础 · 输入',
          items: [
            { text: 'Button 按钮', link: '/components/button' },
            { text: 'Input 输入框', link: '/components/input' },
            { text: 'Select 选择器', link: '/components/select' },
            { text: 'Switch 开关', link: '/components/switch' },
            { text: 'Checkbox 复选框', link: '/components/checkbox' },
            { text: 'Slider 滑块', link: '/components/slider' },
          ],
        },
        {
          text: '基础 · 展示',
          items: [
            { text: 'Badge 徽标', link: '/components/badge' },
            { text: 'Card 卡片', link: '/components/card' },
            { text: 'Avatar 头像', link: '/components/avatar' },
            { text: 'Spinner 加载指示', link: '/components/spinner' },
            { text: 'Progress 进度条', link: '/components/progress' },
            { text: 'SegmentedControl 分段控制', link: '/components/segmented-control' },
            { text: 'List 列表', link: '/components/list' },
          ],
        },
        {
          text: '高级组件',
          items: [
            { text: 'DataTable 数据表格', link: '/components/data-table' },
            { text: 'Form 表单', link: '/components/form' },
            { text: 'Lightbox 灯箱', link: '/components/lightbox' },
          ],
        },
        {
          text: 'AI · 单智能体',
          items: [
            { text: 'Composer AI 输入域', link: '/components/composer' },
            { text: 'AttachmentList 附件列表', link: '/components/attachment-list' },
            { text: 'Thinking 思考过程', link: '/components/thinking' },
            { text: 'ToolExecution 工具执行', link: '/components/tool-execution' },
            { text: 'SkillCard 技能卡片', link: '/components/skill-card' },
            { text: 'WebSearch 联网搜索', link: '/components/web-search' },
            { text: 'WerewolfStage 狼人杀舞台', link: '/components/werewolf-stage' },
            { text: 'SettingsPanel 设置面板', link: '/components/settings-panel' },
            { text: 'Message 消息', link: '/components/message' },
            { text: 'TaskPlan 任务计划', link: '/components/task-plan' },
            { text: 'ApprovalCard 授权确认', link: '/components/approval-card' },
            { text: 'ConversationList 会话列表', link: '/components/conversation-list' },
            { text: 'ErrorBanner 错误横幅', link: '/components/error-banner' },
            { text: 'Toast 轻通知', link: '/components/toast' },
            { text: 'PromptSuggestions 快捷提示', link: '/components/prompt-suggestions' },
            { text: 'Markdown 富文本渲染', link: '/components/markdown' },
            { text: 'Typewriter 打字机', link: '/components/typewriter' },
            { text: 'Usage 用量', link: '/components/usage' },
          ],
        },
        {
          text: 'AI · 多智能体',
          items: [
            { text: 'AgentCard 智能体状态卡', link: '/components/agent-card' },
            { text: 'AgentTeamBoard 团队面板', link: '/components/agent-team-board' },
            { text: 'AgentTree 委派树', link: '/components/agent-tree' },
            { text: 'AgentTimeline 运行时间线', link: '/components/agent-timeline' },
            { text: 'AgentHandoff 交接回执', link: '/components/agent-handoff' },
            { text: 'ApprovalGate 决策门', link: '/components/approval-gate' },
            { text: 'MemoryPanel 记忆面板', link: '/components/memory-panel' },
          ],
        },
      ],
    },
  },
});
