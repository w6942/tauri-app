<template>
  <div class="row">
    <input type="number" v-model="waitTimeMs" placeholder="Enter a name..." />
    <button @click="record">Record</button>
    <button @click="replay">Replay</button>
    <button @click="stop">Stop</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

const waitTimeMs = ref(10);

function record() {
  new WebviewWindow('windowrecord', {
    title: '',
    url: '/record',
    // fullscreen: true,
    transparent: true,
    alwaysOnTop: true,
    // decorations: false,
    skipTaskbar: true,
    focus: true
  });
}
function replay() {
  const value = parseInt(String(waitTimeMs.value));
  const waitTime = Math.max(isNaN(value) ? 1000 : value, 10);
  invoke("start_replay", { waitTimeMs: waitTime });
}

function stop() {
  invoke('stop_replay');
}
</script>