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
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { resourceDir, join } from "@tauri-apps/api/path";
import IconApi from "./assets/api.svg?component";

const workingPath = ref("");

const newlyOpen = ref(false);

var speDataRaw: SpeData;

const speDataShow = ref<SpeData>();

const fileName = ref('');

const frameIndex = ref(0);

const sliceIndex = ref(0);

const silentFlag = ref();
provide('silentlySave', silentFlag);

const messageText = ref('');
const messageType = ref('ok')

function showMessage(text: string, type: 'ok' | 'error') {
  messageText.value = text;
  messageType.value = type;
  setTimeout(() => messageText.value = '', 3000);
}

async function openFromPath(filePath: string) {
  let baseName = await path.basename(filePath);
  invoke("open_file", { path: filePath })
  .then((str) => {
      fileName.value = baseName.substring(0, baseName.length - 4);
    
      workingPath.value = filePath;

      speDataRaw = JSON.parse(str as string) as SpeData;
      speDataShow.value = speDataRaw;
      newlyOpen.value = true;
    }).catch((msg) => {
      showMessage(msg, 'error');
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

async function copyAgentPrompt() {
  let dir = await resourceDir();
  while (dir.endsWith("/") || dir.endsWith("\\")) {
    dir = dir.slice(0, -1);
  }
  const skillPath = await join(dir, "skill", "SKILL.md");
  const prompt = [
    "我已安装 ARSpeViewer（角分辨光谱浏览器），它自带面向 Agent 的 Python 数据接口技能。请按以下步骤完成接入：",
    `1. 读取 "${skillPath}"；`,
    `2. 将文件中所有 <ARSPEVIEWER_INSTALL_DIR> 占位符替换为 "${dir}"；`,
    "3. 将修改后的内容安装为你的技能（保存到你的技能目录，如 ~/.agents/skills/arspe-viewer/SKILL.md，或按你所用的技能安装机制处理）；",
    "4. 安装完成后，我后续提到光谱数据处理需求时请优先使用该技能中的接口。",
  ].join("\n");
  try {
    await writeText(prompt);
    showMessage('接入提示词已复制，请粘贴给你的 AI Agent 完成安装', 'ok');
  } catch (e) {
    showMessage(String(e), 'error');
  }
}

onMounted(async () => {
  let source = (await getMatches()).args.source.value;
  if (typeof source == 'string') {
    openFromPath(source);
    // if (['.spe', '.asc', '.txt', '.csv'].includes(source.substring(source.length - 4))) {
    //   openFromPath(source);
    // }
  }
  listen<{ [key: string]: any }>('tauri://drag-drop', event => {
    let filePath: string = event.payload.paths[0];
    openFromPath(filePath);
    // if (['.spe', '.asc', '.txt', '.csv'].includes(filePath.substring(filePath.length - 4))) {
    //   openFromPath(filePath);
    // }
  });
})

function stretch(eVMode: boolean, xMode: string, tanMin: number, tanMax: number) {
  newlyOpen.value = false;
  if (!eVMode && xMode == 'tan') {
    speDataShow.value = speDataRaw;
  } else {
    invoke('stretch', {
        speStr: JSON.stringify(speDataRaw),
        evMode: eVMode,
        xMode: xMode,
        tanMin: tanMin,
        tanMax: tanMax
    }).then(newData => {
      speDataShow.value = JSON.parse(newData as string);
    }).catch((msg) => {
      showMessage(msg as string, 'error');
    });
  }
}
</script>

<template>
  <nav>
    <button @click="openDataFile" title="打开文件">
      <IconOpen />
    </button>
    <button @click="silentlySaveImage" title="一键导出图片">
      <IconExport />
    </button>
    <button @click="copyAgentPrompt" title="Agent 接入：复制提示词，让 AI Agent 安装随包的数据接口技能">
      <IconApi />
    </button>
  </nav>
  <Teleport to="body">
    <Transition name="message">
      <p ref="message" v-if="messageText != ''" :class="['message', messageType]">{{ messageText }}</p>
    </Transition>
  </Teleport>
  <div id="content" @dragover.prevent="" @drop="dropToOpen">
    <ARSpectrumViewer :data="speDataShow" :name="fileName" :path="workingPath" :newly-open="newlyOpen"
      :hightlight-index="sliceIndex" v-model="frameIndex" @show-message="showMessage"
      @slice-at-index="(index: number) => sliceIndex = index" @stretch="stretch" />
    <SpectrumViewer :data="speDataShow" :name="fileName" :frame-index="frameIndex" v-model="sliceIndex"
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