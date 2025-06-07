<script setup lang="ts">
import { onMounted, provide, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import IconOpen from "./assets/folder-open.svg?component";
import IconExport from './assets/down-picture.svg?component';
import { SpeData } from "./scripts/DataViewer";
import ARSpectrumViewer from "./ARSpectrumViewer.vue";
import { path } from "@tauri-apps/api";
import SpectrumViewer from "./SpectrumViewer.vue";
import { getMatches } from "@tauri-apps/plugin-cli";
import { listen } from "@tauri-apps/api/event";

const workingPath = ref("");

const speData = ref<SpeData>();

const fileName = ref('');

const sliceIndex = ref(0);

const silentFlag = ref();
provide('silentlySave', silentFlag);

const MessageText = ref('');

function showMessage(text: string) {
  MessageText.value = text;
  setTimeout(() => MessageText.value = '', 3000);
}

async function openFromPath(filePath: string) {
  let baseName = await path.basename(filePath);
  fileName.value = baseName.substring(0, baseName.length - 4);

  workingPath.value = filePath;
  invoke("open_file", { path: filePath })
    .then((str) => {
      speData.value = JSON.parse(str as string) as SpeData;
    });
}

async function openDataFile() {
  let filePath = await open({
    multiple: false,
    filters: [{
      name: '文本文件',
      extensions: ['txt', 'csv', 'asc']
    }, {
      name: 'spe文件',
      extensions: ['spe']
    }]
  });

  if (filePath) {
    openFromPath(filePath);
  }
}

async function dropToOpen(e: DragEvent) {
  console.log('drop');
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    let filePath = e.dataTransfer.files[0];
    console.log(filePath);
  }
}

function silentlySaveImage() {
  silentFlag.value = workingPath.value.split(fileName.value)[0];
  document.getElementById("save-slice-spec")?.click();
  document.getElementById("save-ar-spec")?.click();
  silentFlag.value = undefined;
}

onMounted(async () => {
  let source = (await getMatches()).args.source.value;
  if (typeof source == 'string') {
    if (['.spe', '.asc', '.txt', '.csv'].includes(source.substring(source.length - 4))) {
      openFromPath(source);
    }
  }
  listen<{ [key: string]: any }>('tauri://drag-drop', event => {
    let filePath: string = event.payload.paths[0];
    if (['.spe', '.asc', '.txt', '.csv'].includes(filePath.substring(filePath.length - 4))) {
      openFromPath(filePath);
    }
  });
})
</script>

<template>
  <nav>
    <button @click="openDataFile" title="打开文件">
      <IconOpen />
    </button>
    <button @click="silentlySaveImage" title="一键导出图片">
      <IconExport />
    </button>
  </nav>
  <Teleport to="body">
    <Transition name="message">
      <p ref="message" v-if="MessageText != ''" class="message">{{ MessageText }}</p>
    </Transition>
  </Teleport>
  <div id="content" @dragover.prevent="" @drop="dropToOpen">
    <ARSpectrumViewer :data="speData" :name="fileName" :path="workingPath" @show-message="showMessage"
      @slice-at-index="index => sliceIndex = index" />
    <SpectrumViewer :data="speData" :name="fileName" v-model="sliceIndex"
      @show-message="showMessage" />
  </div>
</template>

<style>
.message-enter-active,
.message-leave-active {
  transition: all 0.3s ease;
}

.message-enter-from,
.message-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>