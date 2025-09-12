<template>
  <div @click="record" style="background-color: rgba(0, 0, 0, .5);height: 100%"></div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useEventListener } from '@vueuse/core';


useEventListener('keydown', event => {
  if (event.key === 'Escape') {
    close();
  }
});

function record() {
  invoke('record').finally(() => close());
}

function close() {
  getCurrentWindow().close();
}
</script>