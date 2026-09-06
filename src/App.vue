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
import { getVersion } from "@tauri-apps/api/app";
import { openUrl } from "@tauri-apps/plugin-opener";
import IconApi from "./assets/api.svg?component";
import IconInfo from "./assets/info.svg?component";
import IconGithub from "./assets/github.svg?component";
import IconUpdate from "./assets/update.svg?component";

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

// ---- 关于页与版本检测 ----

const REPO_URL = "https://github.com/prcxhy/ARSpeViewer";
const ISSUES_URL = REPO_URL + "/issues";
const RELEASES_URL = REPO_URL + "/releases/latest";
const RELEASES_API = "https://api.github.com/repos/prcxhy/ARSpeViewer/releases/latest";
const CHECK_KEY = "arspeviewer.update-check";

const aboutOpen = ref(false);
const appVersion = ref("");
const latestVersion = ref<string | null>(null);
const hasUpdate = ref(false);
const checkFailed = ref(false);

const THANKS: { group: string; items: Array<[string, string]> }[] = [
  {
    group: "框架",
    items: [
      ["Tauri", "https://github.com/tauri-apps/tauri"],
      ["Vue", "https://github.com/vuejs/core"],
      ["Vite", "https://github.com/vitejs/vite"],
      ["TypeScript", "https://github.com/microsoft/TypeScript"],
      ["Rust", "https://github.com/rust-lang/rust"],
    ],
  },
  {
    group: "数据与功能库",
    items: [
      ["ECharts", "https://github.com/apache/echarts"],
      ["serde", "https://github.com/serde-rs/serde"],
      ["quick-xml", "https://github.com/tafia/quick-xml"],
      ["ndarray", "https://github.com/rust-ndarray/ndarray"],
      ["ninterp", "https://github.com/NatLabRockies/ninterp"],
      ["rayon", "https://github.com/rayon-rs/rayon"],
      ["PyO3", "https://github.com/PyO3/pyo3"],
      ["maturin", "https://github.com/PyO3/maturin"],
      ["Tauri 官方插件", "https://github.com/tauri-apps/plugins-workspace"],
    ],
  },
];

function isNewer(latest: string, current: string): boolean {
  const l = latest.split(".").map(Number);
  const c = current.split(".").map(Number);
  for (let i = 0; i < 3; i++) {
    if ((l[i] ?? 0) !== (c[i] ?? 0)) return (l[i] ?? 0) > (c[i] ?? 0);
  }
  return false;
}

async function checkUpdate(force: boolean) {
  if (!appVersion.value) {
    appVersion.value = await getVersion();
  }
  if (!force) {
    try {
      const cache = JSON.parse(localStorage.getItem(CHECK_KEY) ?? "") as {
        checkedAt: number;
        latest: string | null;
      };
      if (Date.now() - cache.checkedAt < 24 * 3600 * 1000) {
        latestVersion.value = cache.latest;
        hasUpdate.value = cache.latest !== null && isNewer(cache.latest, appVersion.value);
        checkFailed.value = cache.latest === null;
        return;
      }
    } catch {
      // 缓存缺失或损坏 → 走完整检测
    }
  }
  let latest: string | null = null;
  try {
    const res = await fetch(RELEASES_API);
    if (res.ok) {
      const data = await res.json();
      const match = typeof data.tag_name === "string"
        ? data.tag_name.match(/^app-v(\d+\.\d+\.\d+)$/)
        : null;
      if (match) {
        latest = match[1];
      }
    }
    checkFailed.value = latest === null;
  } catch {
    checkFailed.value = true;
  }
  // 失败也写 checkedAt：当天不再重试，防 GitHub API 限流（60 次/小时）
  localStorage.setItem(CHECK_KEY, JSON.stringify({ checkedAt: Date.now(), latest }));
  latestVersion.value = latest;
  hasUpdate.value = latest !== null && isNewer(latest, appVersion.value);
}

function openAbout() {
  aboutOpen.value = true;
  if (checkFailed.value) {
    checkUpdate(true);
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
  appVersion.value = await getVersion();
  checkUpdate(false);
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
    <button class="about-button" @click="openAbout" title="关于 ARSpeViewer">
      <IconInfo />
      <span v-if="hasUpdate" class="update-badge"></span>
    </button>
  </nav>
  <Teleport to="body">
    <Transition name="message">
      <p ref="message" v-if="messageText != ''" :class="['message', messageType]">{{ messageText }}</p>
    </Transition>
  </Teleport>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="aboutOpen" class="about-mask" @click.self="aboutOpen = false">
        <div class="about-card">
          <button class="about-close" @click="aboutOpen = false" title="关闭">✕</button>
          <h2>ARSpeViewer <small>v{{ appVersion }}</small></h2>
          <p class="about-desc">角分辨光谱浏览器（Angle-Resolved Spectral Viewer）</p>
          <p class="about-links">
            <button class="link" @click="openUrl(REPO_URL)"><IconGithub />仓库</button>
            <button class="link" @click="openUrl(ISSUES_URL)"><IconGithub />Issues</button>
            <button class="link" @click="openUrl(RELEASES_URL)"><IconGithub />Releases</button>
            <span>GPL-3.0 License</span>
          </p>
          <div class="about-update">
            <p v-if="hasUpdate" class="update-line">
              <IconUpdate />
              <span>新版本 {{ latestVersion }} 可用</span>
              <button class="link" @click="openUrl(RELEASES_URL)">前往下载更新</button>
            </p>
            <p v-else-if="!checkFailed">已是最新版本</p>
            <p v-else>未能检测更新（可能离线或达到 GitHub API 限额，稍后自动重试）</p>
          </div>
          <h3>开源致谢</h3>
          <div class="about-thanks">
            <p v-for="g in THANKS" :key="g.group">
              <b>{{ g.group }}：</b>
              <button v-for="[name, url] in g.items" :key="name" class="link"
                @click="openUrl(url)">{{ name }}</button>
            </p>
            <p>以及所有间接依赖的开源项目。</p>
          </div>
        </div>
      </div>
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

.about-button {
  position: relative;
}

.update-badge {
  position: absolute;
  top: 1mm;
  right: 1mm;
  width: 2mm;
  height: 2mm;
  border-radius: 50%;
  background-color: #2eaf64;
}

.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.about-mask {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(0, 0, 0, 0.35);
}

.about-card {
  position: relative;
  min-width: 90mm;
  max-width: 120mm;
  padding: 6mm 8mm;
  border-radius: 2mm;
  background-color: #fff;
  box-shadow: 0 2mm 8mm rgba(0, 0, 0, 0.25);
}

.about-card h2 {
  margin: 0 0 1mm;
}

.about-card h2 small {
  font-size: 0.6em;
  font-weight: normal;
  color: #666;
}

.about-close {
  position: absolute;
  top: 2mm;
  right: 2mm;
  padding: 0 1mm;
  border: none;
  background: none;
  font-size: 4mm;
  color: #666;
  cursor: pointer;
}

.about-desc {
  margin: 0 0 2mm;
  color: #444;
}

.about-links {
  display: flex;
  flex-wrap: wrap;
  gap: 3mm;
  align-items: center;
  margin: 0 0 3mm;
}

button.link {
  display: inline-flex;
  align-items: center;
  gap: 1mm;
  padding: 0;
  border: none;
  background: none;
  color: #2f6fde;
  cursor: pointer;
  font-size: 3.2mm;
}

button.link:hover {
  text-decoration: underline;
}

.about-links svg,
.update-line svg {
  width: 3.5mm;
  height: 3.5mm;
}

.about-update {
  margin: 0 0 3mm;
  padding: 2mm 3mm;
  border-radius: 1mm;
  background-color: var(--color-bg-7, #f3f3f3);
}

.about-update p {
  margin: 0;
}

.update-line {
  display: flex;
  align-items: center;
  gap: 2mm;
}

.about-thanks {
  font-size: 3.2mm;
  line-height: 2;
  color: #444;
}

.about-thanks p {
  margin: 0;
}
</style>