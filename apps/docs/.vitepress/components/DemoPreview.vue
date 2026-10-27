<script setup lang="ts">
import type { Root } from 'react-dom/client';
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { demos } from '../../demos';

/**
 * React demo 桥：VitePress（Vue）页面内挂载 React 组件实例。
 * 用法：<DemoPreview name="ButtonBasic" />
 * demos/index.ts 登记所有示例（组件唯一真源），源码展示由 md 的 <<< 嵌入语法完成。
 */
const props = defineProps<{ name: string }>();
const host = ref<HTMLElement | null>(null);
let root: Root | null = null;

async function mount() {
  const el = host.value;
  const Comp = demos[props.name];
  if (!el || !Comp) return;
  const [{ createElement }, { createRoot }] = await Promise.all([
    import('react'),
    import('react-dom/client'),
  ]);
  root = createRoot(el);
  root.render(createElement(Comp));
}

function unmount() {
  root?.unmount();
  root = null;
}

onMounted(mount);
watch(
  () => props.name,
  () => {
    unmount();
    mount();
  },
);
onBeforeUnmount(unmount);
</script>

<template>
  <div class="demo-preview">
    <div class="demo-stage">
      <div ref="host"></div>
    </div>
  </div>
</template>
