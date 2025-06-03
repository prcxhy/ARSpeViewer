<script setup lang="ts">
import { nextTick, useTemplateRef, watch } from 'vue';
import IconCopy from './assets/clipboard.svg?component';
import IconExport from './assets/export.svg?component';
import { drawSpec, saveImage, SpeData } from './scripts/DataViewer';
import * as echarts from 'echarts';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const prop = defineProps<{
    data: SpeData
    name: string
}>()

var chart2: echarts.ECharts;
const spec = useTemplateRef('spec');

const sliceIndex = defineModel<number>({ default: 0 });

watch(() => prop.data, newData => {
    if (chart2) {
        chart2.dispose();
    }
    nextTick().then(() => {
        chart2 = echarts.init(spec.value! as HTMLDivElement);
        if (newData) {
            drawSpec(chart2, newData, sliceIndex.value, prop.name);
        }
    })
}, { immediate: true });

watch(sliceIndex, newIndex => {
    if (chart2) {
        chart2.dispose();
    }
    nextTick().then(() => {
        chart2 = echarts.init(spec.value! as HTMLDivElement);
        drawSpec(chart2, prop.data, newIndex, prop.name);
    })
}, { immediate: true });

const emit = defineEmits(['copy-to-clipboard']);

function copyToClipboard() {
    emit('copy-to-clipboard', '数据已复制到剪贴板，可在Origin直接粘贴表格');

    let str = 'Wavelength\tcounts\nnm\n\n';
    prop.data.wavelength.forEach((lambda, j) => {
        str += `${lambda}\t${prop.data.frame[0][sliceIndex.value * prop.data.width + j]}\n`;
    });
    writeText(str);
}
</script>

<template>
    <div id="spec-container">
        <div ref="spec" id="spectrum"></div>
        <div id="slice-index-input">
            <label for="slice-index">横轴索引</label>
            <input type="range" :min="0" :max="prop.data.height - 1" v-model.number="sliceIndex">
            <input id="slice-index" type="number" v-model.number="sliceIndex">
        </div>
        <button style="grid-column-start: 3;" @click="copyToClipboard" title="复制数据到剪贴板">
            <IconCopy />
        </button>
        <button style="grid-column-start: 4;" @click="saveImage(chart2, `切片光谱-${prop.name}`)" title="导出图片">
            <IconExport />
        </button>
    </div>
</template>

<style>
#spec-container {
    padding: 2mm;
    gap: 1mm;
    background-color: rgb(223, 223, 223);
    border-radius: 3mm;
    display: grid;
    grid-template-rows: 1fr auto;
    grid-template-columns: auto 1fr auto auto;
    transition: 0.3s;
}

#spec-container:hover {
    filter: drop-shadow(0px 0px 4px rgba(0, 0, 0, 0.1))
}

#spectrum {
    width: 16cm;
    height: 12cm;
    /* margin: 0px auto; */
    background-color: white;
    border-radius: 1mm;
    grid-column: 1 / -1;
    justify-self: center;
    align-self: center;
}

#slice-index-input {
    display: flex;
    gap: 2mm;
    padding-left: 1mm;
    align-items: center;
}
</style>