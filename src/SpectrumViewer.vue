<script setup lang="ts">
import { inject, nextTick, Ref, useTemplateRef, watch } from 'vue';
import IconCopy from './assets/clipboard.svg?component';
import IconExport from './assets/down-picture.svg?component';
import { drawSpec, saveImage, SpeData } from './scripts/DataViewer';
import * as echarts from 'echarts';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const prop = defineProps<{
    data?: SpeData
    name: string
}>()
const silentPath: Ref<string> | undefined = inject('silentlySave');

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
        if (prop.data) {
            chart2 = echarts.init(spec.value! as HTMLDivElement);
            drawSpec(chart2, prop.data, newIndex, prop.name);
        }
    })
}, { immediate: true });

const emit = defineEmits(['show-message']);

function copyToClipboard() {
    if (!prop.data) { return }

    emit('show-message', '数据已复制到剪贴板，可在Origin直接粘贴表格');
    let str = prop.data!.wavelength ? 'Wavelength\tcounts\nnm\n\n' : 'counts\n\n\n';

    let key = Object.keys(prop.data!.frame[0])[0];
    if (prop.data?.wavelength) {
        prop.data.wavelength.forEach((lambda, j) => {
            str += `${lambda}\t${prop.data?.frame[0][key][sliceIndex.value * prop.data!.width + j]}\n`;
        })
    } else {
        let start = sliceIndex.value * prop.data!.width;
        str += prop.data?.frame[0][key].slice(start, start + prop.data!.width).join('\n');
    }
    writeText(str);
}
</script>

<template>
    <div id="spec-container">
        <div ref="spec" id="spectrum"></div>
        <div id="slice-index-input">
            <label for="slice-index">横轴索引</label>
            <input v-if="prop.data" type="range" :min="0" :max="prop.data.height - 1" v-model.number="sliceIndex">
            <input id="slice-index" type="number" v-model.number="sliceIndex">
        </div>
        <button style="grid-column-start: 3;" @click="copyToClipboard" title="复制数据到剪贴板">
            <IconCopy />
        </button>
        <button id="save-slice-spec" style="grid-column-start: 4;"
        @click="saveImage(chart2, `切片光谱-${prop.name}`, !prop.data, silentPath).then(msg => {
            if (msg) { emit('show-message', msg) }
        })" title="导出图片">
            <IconExport />
        </button>
    </div>
</template>

<style>
#spec-container {
    padding: 2mm;
    gap: 2mm;
    background-color: var(--color-bg-9);
    /* border: 2px solid var(--color-bg-9); */
    border-radius: 3mm;
    display: grid;
    grid-template-rows: 1fr auto;
    grid-template-columns: auto 1fr auto auto;
    transition: 0.3s;
    filter: drop-shadow(0px 0px 4px rgba(0, 0, 0, 0.1))
}

#spec-container:hover {
    /* background: 
        linear-gradient(var(--color-bg-9), var(--color-bg-9)) padding-box,
        linear-gradient(-45deg, rgba(240,249,33,1) 0%, rgba(253,153,39,1) 20%, rgba(223,55,83,1) 40%, rgba(158,6,152,1) 60%, rgba(83,1,168,1) 80%, rgba(13,8,135,1) 100%) border-box;
    border: 2px solid transparent; */
    filter: drop-shadow(0px 0px 4px rgba(0, 0, 0, 0.2))
}

#spectrum {
    width: 16cm;
    height: 12cm;
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