<script setup lang="ts">
import { computed, nextTick, onMounted, ref, useTemplateRef, watch } from 'vue';
import * as echarts from 'echarts';
import { downSampling, drawARSpec, SpeData, saveImage, CONST_1240 } from './scripts/DataViewer';
import ModeSwitch from './components/ModeSwitch.vue';
import IconOption from './assets/config.svg?component';
import IconArrow from './assets/right.svg?component';
import IconCopy from './assets/clipboard.svg?component';
import IconExport from './assets/export.svg?component';
import IconReset from './assets/undo.svg?component';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const prop = defineProps<{
    data?: SpeData
    name: string
    path: string
}>()

var chart1: echarts.ECharts;
const arspc = useTemplateRef('arspc');

const dataSnapshot = computed(() => {
    if (prop.data) {
        return downSampling(prop.data, 3);
    }
})

const emit = defineEmits(['copy-to-clipboard', 'slice-at-index']);

const xMinIndex = ref();
const xMaxIndex = ref();
const xMinAngle = ref();
const xMaxAngle = ref();

const yAxisMode = ref(true);
const yMinLambda = ref();
const yMaxLambda = ref();
const yMinEnergy = computed({
    get: () => CONST_1240 / yMaxLambda.value,
    set: val => {
        yMaxLambda.value = CONST_1240 / val;
    }
});
const yMaxEnergy = computed({
    get: () => CONST_1240 / yMinLambda.value,
    set: val => {
        yMinLambda.value = CONST_1240 / val;
    }
});
const yMinIndex = computed(() => {
    if (dataSnapshot.value) {
        let min = dataSnapshot.value!.wavelength[0];
        let max = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
        return Math.round((yMinLambda.value - min) / (max - min) * (dataSnapshot.value!.width - 1));
    }
});
const yMaxIndex = computed(() => {
    if (dataSnapshot.value) {
        let min = dataSnapshot.value!.wavelength[0];
        let max = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
        return Math.round((yMaxLambda.value - min) / (max - min) * (dataSnapshot.value!.width - 1));
    }
});

onMounted(() => {
    chart1 = echarts.init(arspc.value! as HTMLDivElement);

    chart1!.on('click', (params: { [key: string]: any }) => {
        if (params.componentType == 'series') {
            let index = Math.round(params.data[0] / (dataSnapshot.value!.height - 1) * (prop.data!.height - 1))
            emit('slice-at-index', index);
        }
    })
})

watch(() => prop.data, newData => {
    nextTick(() => {
        if (newData) {
            drawARSpec(chart1, dataSnapshot.value!, prop.name, 0, prop.data!.height - 1);
            yAxisMode.value = true;
            xMinIndex.value = 0;
            xMaxIndex.value = newData!.height - 1;
            yMinLambda.value = newData!.wavelength[0];
            yMaxLambda.value = newData!.wavelength[newData!.width - 1];
        }
    })
}, { immediate: true });

const xAxisIsTan = computed(() => {
    let yes = xMinAngle.value < xMaxAngle.value
        && xMinAngle.value != undefined
        && xMaxAngle.value != undefined
        && xMinAngle.value != ''
        && xMaxAngle.value != ''
    return yes
})

watch(xAxisIsTan, isTan => {
    let xAxisOptions = (chart1.getOption().xAxis as { [key: string]: any }[]);
    xAxisOptions[1].show = !isTan;
    xAxisOptions[2].show = isTan;
    if (isTan) {
        xAxisOptions[2].min = Math.tan(xMinAngle.value / 180 * Math.PI);
        xAxisOptions[2].max = Math.tan(xMaxAngle.value / 180 * Math.PI);
    }
    chart1.setOption({
        xAxis: xAxisOptions
    })
})

watch(xMinIndex, newIndex => {
    let min = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value!.height - 1));
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 0,
        startValue: min,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 2,
        startValue: newIndex,
    });
});

watch(xMaxIndex, newIndex => {
    let max = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value!.height - 1));
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 0,
        endValue: max,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 2,
        endValue: newIndex,
    });
});

watch(xMinAngle, newAngle => {
    if (xAxisIsTan.value) {
        let xAxisOptions = (chart1.getOption().xAxis as { [key: string]: any }[]);
        xAxisOptions[2].min = Math.tan(newAngle / 180 * Math.PI);
        chart1.setOption({ xAxis: xAxisOptions });
    }
});

watch(xMaxAngle, newAngle => {
    if (xAxisIsTan.value) {
        let xAxisOptions = (chart1.getOption().xAxis as { [key: string]: any }[]);
        xAxisOptions[2].max = Math.tan(newAngle / 180 * Math.PI);
        chart1.setOption({ xAxis: xAxisOptions });
    }
});

watch(yMinIndex, newIndex => {
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 1,
        startValue: newIndex,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 3,
        startValue: yMinLambda.value,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 4,
        startValue: yMaxEnergy.value,
    });
})

watch(yMaxIndex, newIndex => {
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 1,
        endValue: newIndex,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 3,
        endValue: yMaxLambda.value,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 4,
        endValue: yMinEnergy.value,
    });
})

watch(yAxisMode, isWavelength => {
    let yAxisOptions = (chart1.getOption().yAxis as { [key: string]: any }[]);
    yAxisOptions[0].inverse = !isWavelength;
    yAxisOptions[1].show = isWavelength;
    yAxisOptions[2].show = !isWavelength;
    chart1.setOption({
        yAxis: yAxisOptions
    });
})

const showOptions = ref(false);

function resetYRange() {
    yMinLambda.value = dataSnapshot.value!.wavelength[0];
    yMaxLambda.value = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
}
function resetXRange() {
    xMinIndex.value = 0;
    xMaxIndex.value = prop.data!.height - 1;
}
function resetXBinding() {
    xMinAngle.value = '';
    xMaxAngle.value = '';
}

function copyToClipboard() {
    if (!prop.data) { return }

    emit('copy-to-clipboard', '数据已复制到剪贴板，可在Origin直接粘贴表格');

    let str = `${xAxisIsTan.value ? 'tan(θ)' : 'Index'}\t${yAxisMode.value ? 'Wavelength' : 'Energy'}\tcounts
                \t${yAxisMode.value ? 'nm' : 'eV'}\n\n`;
    if (xAxisIsTan.value) {
        let minTan = Math.tan(xMinAngle.value / 180 * Math.PI);
        let maxTan = Math.tan(xMaxAngle.value / 180 * Math.PI);
        let height = xMaxIndex.value - xMinIndex.value + 1;
        let d = (maxTan - minTan) / (height - 1);
        for (var a = 0; a < height; a++) {
            str += `\t${minTan + a * d}`;
        }
    } else {
        for (var b = xMinIndex.value; b <= xMaxIndex.value; b++) {
            str += `\t${b}`;
        }
    }
    str += '\n';
    let min = prop.data!.wavelength[0];
    let max = prop.data!.wavelength[prop.data!.width - 1];
    let yMin = Math.round((yMinLambda.value - min) / (max - min) * (prop.data!.width - 1));
    let yMax = Math.round((yMaxLambda.value - min) / (max - min) * (prop.data!.width - 1));
    for (var i = yMin; i <= yMax; i++) {
        str += `${yAxisMode.value ? prop.data!.wavelength[i] : (CONST_1240 / prop.data!.wavelength[i])}`;
        for (var j = xMinIndex.value; j <= xMaxIndex.value; j++) {
            str += `\t${prop.data!.frame[0][j * prop.data!.width + i]}`;
        }
        str += '\n';
    }
    writeText(str);
}
</script>

<template>
    <div id="arspc-container">
        <p style="grid-column: 1 / 3;">{{ prop.path }}</p>
        <button @click="showOptions = !showOptions" title="选项">
            <IconOption v-if="!showOptions" />
            <IconArrow v-if="showOptions" />
        </button>
        <div v-show="!prop.data" id="heatmap-placeholder">
            <h3>↖<br>&nbsp;&nbsp;&nbsp;&nbsp;点击打开文件，或将*.spe文件拖拽到窗口内</h3>
        </div>
        <div v-show="prop.data" ref="arspc" id="ar-spectrum"></div>
        <button style="grid-column-start: 2;" @click="copyToClipboard" title="复制数据到剪贴板">
            <IconCopy />
        </button>
        <button style="grid-column-start: 3;" @click="saveImage(chart1, `角分辨光谱-${prop.name}`, !prop.data)" title="导出图片">
            <IconExport />
        </button>
        <Transition>
            <div id="arspc-options" v-if="showOptions">
                <ModeSwitch :name="'纵轴模式'" :mode1="'波长'" :mode2="'能量'" v-model="yAxisMode" />
                <div class="range-input-lambda" v-show="yAxisMode">
                    <label for="y-min-lambda">纵轴范围</label>
                    <button @click="resetYRange" title="重置">
                        <IconReset />
                    </button>
                    <input id="y-min-lambda" type="text" v-model.number="yMinLambda">
                    <label for="y-max-lambda">~</label>
                    <input id="y-max-lambda" type="text" v-model.number="yMaxLambda">
                </div>
                <div class="range-input-energy" v-show="!yAxisMode">
                    <label for="y-min-energy">纵轴范围</label>
                    <button @click="resetYRange" title="重置">
                        <IconReset />
                    </button>
                    <input id="y-min-energy" type="text" v-model.number="yMinEnergy">
                    <label for="y-max-energy">~</label>
                    <input id="y-max-energy" type="text" v-model.number="yMaxEnergy">
                </div>
                <div class="range-input-x">
                    <label for="x-min-index">横轴范围</label>
                    <button @click="resetXRange" title="重置">
                        <IconReset />
                    </button>
                    <input id="x-min-index" type="number" v-model.number="xMinIndex">
                    <label for="x-max-index">~</label>
                    <input id="x-max-index" type="number" v-model.number="xMaxIndex">
                    <p>(索引)</p>
                    <label for="x-min-angle" style="grid-column-start: 1">横轴绑定</label>
                    <button @click="resetXBinding" title="重置">
                        <IconReset />
                    </button>
                    <input id="x-min-angle" type="text" v-model.number="xMinAngle">
                    <label for="x-max-angle">°</label>
                    <input id="x-max-angle" type="text" v-model.number="xMaxAngle">
                    <p>°</p>
                </div>
            </div>
        </Transition>
    </div>
</template>

<style>
#arspc-container {
    padding: 2mm;
    gap: 2mm;
    background-color: var(--color-bg-9);
    /* border: 2px solid var(--color-bg-9); */
    border-radius: 3mm;
    display: grid;
    grid-template-columns: 1fr auto auto;
    grid-template-rows: auto 1fr auto;
    grid-auto-columns: auto;
    transition: 0.3s;
    filter: drop-shadow(0px 0px 4px rgba(0, 0, 0, 0.1))
}

#arspc-container:hover {
    /* background:
        linear-gradient(var(--color-bg-9), var(--color-bg-9)) padding-box,
        linear-gradient(-45deg, rgba(240, 249, 33, 1) 0%, rgba(253, 153, 39, 1) 20%, rgba(223, 55, 83, 1) 40%, rgba(158, 6, 152, 1) 60%, rgba(83, 1, 168, 1) 80%, rgba(13, 8, 135, 1) 100%) border-box;
    border: 2px solid transparent; */
    filter: drop-shadow(0px 0px 4px rgba(0, 0, 0, 0.2))
}

/* #arspc-container:hover::after {
    content: '●';
    color: rgb(253, 153, 39);
    align-self: self-end;
    font-size: 3mm;
    grid-row: 3 / 4;
    grid-column: 1 / 2;
} */

#heatmap-placeholder,
#ar-spectrum {
    background-color: white;
    width: 12cm;
    height: 16cm;
    border-radius: 1mm;
    grid-row: 2 / 3;
    grid-column: 1 / 4;
    align-self: center;
}

.v-enter-active,
.v-leave-active {
    transition: all 0.3s ease;
}

.v-enter-from,
.v-leave-to {
    opacity: 0;
    transform: translateX(-10px);
}

#arspc-options {
    display: flex;
    grid-row: 2 / 3;
    grid-column: 4 / 5;
    flex-direction: column;
    padding: 0px 1mm;
    justify-content: left;
    gap: 1mm;
}

.range-input-lambda,
.range-input-energy {
    display: flex;
    gap: 1mm;
    justify-content: left;
    align-items: center;
}

.range-input-x {
    display: grid;
    grid-template-columns: repeat(6, auto);
    grid-row: repeat(2, auto);
    gap: 1mm;
    justify-content: left;
    align-items: center;
}

.range-input-x p {
    user-select: none;
}
</style>