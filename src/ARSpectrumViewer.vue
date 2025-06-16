<script setup lang="ts">
import { computed, inject, nextTick, onMounted, Ref, ref, useTemplateRef, watch } from 'vue';
import * as echarts from 'echarts';
import { downSampling, drawARSpec, SpeData, saveImage, CONST_1240, compatible } from './scripts/DataViewer';
import Pagination from './components/Pagination.vue';
import IconCopy from './assets/clipboard.svg?component';
import IconExport from './assets/down-picture.svg?component';
import IconOpen from './assets/folder-open.svg?component';
import IconReset from './assets/undo.svg?component';
import IconLock from './assets/lock.svg?component';
import IconUnlock from './assets/unlock.svg?component';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const prop = defineProps<{
    data?: SpeData
    name: string
    path: string
}>()

const frameIndex = defineModel<number>({ default: 0 });

const focusing = ref(false);

const silentPath: Ref<string> | undefined = inject('silentlySave');

var chart1: echarts.ECharts;
const arspc = useTemplateRef('arspc');

const dataSnapshot = computed(() => {
    if (prop.data) {
        return downSampling(prop.data, [chart1.getWidth() - 168, chart1.getHeight() - 84]);
        // return downSampling(prop.data, [1024, 1024]);
    }
})

const emit = defineEmits(['show-message', 'slice-at-index', 'stretch']);

const xMinIndex = ref();
const xMaxIndex = ref();
const bindingLock = ref(false);
const xMode = ref('tan');
var X_RANGE_BUFFER = [0, 0];
const xMinInput = ref();
const xMaxInput = ref();

const eVMode = ref(false);
var EV_MODE = false;
const yMinInput = ref();
const yMaxInput = ref();

const yMinIndex = computed(() => {
    if (dataSnapshot.value) {
        if (dataSnapshot.value.wavelength && !EV_MODE) {
            let min = dataSnapshot.value.wavelength[0];
            let max = dataSnapshot.value.wavelength[dataSnapshot.value!.width - 1];
            return Math.round((yMinInput.value - min) / (max - min) * (dataSnapshot.value.width - 1));
        } else if (dataSnapshot.value.wavelength && EV_MODE) {
            let max = CONST_1240 / dataSnapshot.value.wavelength[0];
            let min = CONST_1240 / dataSnapshot.value.wavelength[dataSnapshot.value!.width - 1];
            return Math.round((max - yMaxInput.value) / (max - min) * (dataSnapshot.value.width - 1));
        } else {
            return Math.round(yMinInput.value / (prop.data!.width - 1) * (dataSnapshot.value.width - 1));
        }
    }
});
const yMaxIndex = computed(() => {
    if (dataSnapshot.value) {
        if (dataSnapshot.value.wavelength && !EV_MODE) {
            let min = dataSnapshot.value.wavelength[0];
            let max = dataSnapshot.value.wavelength[dataSnapshot.value!.width - 1];
            return Math.round((yMaxInput.value - min) / (max - min) * (dataSnapshot.value.width - 1));
        } else if (dataSnapshot.value.wavelength && EV_MODE) {
            let max = CONST_1240 / dataSnapshot.value.wavelength[0];
            let min = CONST_1240 / dataSnapshot.value.wavelength[dataSnapshot.value!.width - 1];
            return Math.round((max - yMinInput.value) / (max - min) * (dataSnapshot.value.width - 1));
        } else {
            return Math.round(yMaxInput.value / (prop.data!.width - 1) * (dataSnapshot.value.width - 1));
        }
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

watch(() => prop.data, (newData, oldData) => {
    nextTick(() => {
        if (newData) {
            frameIndex.value = 0;
            if (!oldData) {
                drawARSpec(chart1, dataSnapshot.value!, {
                    name: prop.name, frameIndex: frameIndex.value,
                    eVMode: false, xMode: '',
                    yMin: 0, yMax: prop.data!.width - 1,
                    xMinIndex: 0, xMaxIndex: prop.data!.height - 1
                });
                eVMode.value = false;
                xMinIndex.value = 0;
                xMaxIndex.value = newData!.height - 1;
                yMinInput.value = newData!.wavelength ? newData!.wavelength[0] : 0;
                yMaxInput.value = newData!.wavelength ? newData!.wavelength[newData!.width - 1] : newData!.width - 1;
                inputNA.value = '';
            } else {
                let info = compatible(newData, oldData, xMinIndex.value, xMaxIndex.value, yMinInput.value, yMaxInput.value);
                if (!(newData.wavelength && oldData.wavelength)) {
                    eVMode.value = false;
                }
                yMinInput.value = eVMode.value ? CONST_1240 / info.yMax : info.yMin;
                yMaxInput.value = eVMode.value ? CONST_1240 / info.yMin : info.yMax;
                xMinIndex.value = info.xMinIndex;
                xMaxIndex.value = info.xMaxIndex;
                if (!info.xCompate) {
                    bindingLock.value = false;
                }
                if (!info.yCompate && info.xCompate && xMode.value == 'k') {
                    xMode.value = 'tan';
                } else {
                    xMinInput.value = X_RANGE_BUFFER[0];
                    xMaxInput.value = X_RANGE_BUFFER[1];
                }
                drawARSpec(chart1, dataSnapshot.value!, {
                    name: prop.name, frameIndex: frameIndex.value,
                    eVMode: eVMode.value, xMode: xMode.value,
                    yMin: 0, yMax: prop.data!.width - 1,
                    xMinIndex: 0, xMaxIndex: prop.data!.height - 1,
                    xMin: tanMin.value, xMax: tanMax.value
                });
            }
        }
    })
}, { immediate: true });

watch(frameIndex, newIndex => {
    drawARSpec(chart1, dataSnapshot.value!, {
        name: prop.name, frameIndex: newIndex,
        eVMode: eVMode.value, xMode: bindingLock.value ? xMode.value : '',
        yMin: 0, yMax: prop.data!.width - 1,
        xMinIndex: 0, xMaxIndex: prop.data!.height - 1,
        xMin: tanMin.value, xMax: tanMax.value
    });
})

const inputNA = ref();
const tanMin = ref();
const tanMax = ref();

watch(bindingLock, locked => {
    let tan = Math.tan(Math.asin(inputNA.value));
    if (locked) {
        xMinInput.value = -tan;
        xMaxInput.value = tan;
        let a = xMaxIndex.value - xMinIndex.value;
        tanMin.value = -tan - xMinIndex.value / a * 2 * tan;
        tanMax.value = tan + (prop.data!.height - 1 - xMaxIndex.value) / a * 2 * tan;
    } else {
        X_RANGE_BUFFER = [-tan, tan];
        xMode.value = 'tan';
    }
    let xAxisOptions = (chart1.getOption().xAxis as { [key: string]: any }[]);
    xAxisOptions[1].show = !locked;
    xAxisOptions[2].show = locked;
    xAxisOptions[2].min = -tan;
    xAxisOptions[2].max = tan;
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

watch(xMinInput, newVal => {
    let min = 0;
    if (xMode.value == 'tan') {
        min = Math.round((newVal - tanMin.value) / (tanMax.value - tanMin.value) * (dataSnapshot.value!.height - 1));
    } else if (xMode.value == 'angle') {
        let minAngle = Math.atan(tanMin.value) / Math.PI * 180;
        let maxAngle = Math.atan(tanMax.value) / Math.PI * 180;
        min = Math.round((newVal - minAngle) / (maxAngle - minAngle) * (dataSnapshot.value!.height - 1));
    } else {
        let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
        let minK = 2 * Math.PI * tanMin.value / lambda * 1000;
        let maxK = 2 * Math.PI * tanMax.value / lambda * 1000;
        min = Math.round((newVal - minK) / (maxK - minK) * (dataSnapshot.value!.height - 1));
    }
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 0,
        startValue: min,
    });
    // chart1.dispatchAction({
    //     type: 'dataZoom',
    //     dataZoomIndex: 2,
    //     startValue: newVal,
    // });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 5,
        startValue: newVal,
    });
});

watch(xMaxInput, newVal => {
    let max = 0;
    if (xMode.value == 'tan') {
        max = Math.round((newVal - tanMin.value) / (tanMax.value - tanMin.value) * (dataSnapshot.value!.height - 1));
    } else if (xMode.value == 'angle') {
        let minAngle = Math.atan(tanMin.value) / Math.PI * 180;
        let maxAngle = Math.atan(tanMax.value) / Math.PI * 180;
        max = Math.round((newVal - minAngle) / (maxAngle - minAngle) * (dataSnapshot.value!.height - 1));
    } else {
        let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
        let minK = 2 * Math.PI * tanMin.value / lambda * 1000;
        let maxK = 2 * Math.PI * tanMax.value / lambda * 1000;
        max = Math.round((newVal - minK) / (maxK - minK) * (dataSnapshot.value!.height - 1));
    }
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 0,
        endValue: max,
    });
    // chart1.dispatchAction({
    //     type: 'dataZoom',
    //     dataZoomIndex: 2,
    //     endValue: newVal,
    // });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 5,
        endValue: newVal,
    });
});

watch(xMode, (newMode, oldMode) => {
    let newMin = 0;
    let newMax = 0;
    if (newMode == 'angle' && oldMode == 'tan') {
        newMin = Math.atan(xMinInput.value) / Math.PI * 180;
        newMax = Math.atan(xMaxInput.value) / Math.PI * 180;
    }
    // let lambda = eVMode.value ? CONST_1240 / yMinInput.value : yMaxInput.value;
    // let lambda = EV_MODE ? CONST_1240 / yMaxInput.value : yMinInput.value;
    if (newMode == 'k' && oldMode == 'tan') {
        let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
        newMin = 2 * Math.PI * xMinInput.value / lambda * 1000;
        newMax = 2 * Math.PI * xMaxInput.value / lambda * 1000;
    }
    if (oldMode == 'angle') {
        newMin = Math.tan(xMinInput.value / 180 * Math.PI);
        newMax = Math.tan(xMaxInput.value / 180 * Math.PI);
        if (newMode == 'k') {
            let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
            newMin /= lambda / (1000 * 2 * Math.PI);
            newMax /= lambda / (1000 * 2 * Math.PI);
        }
    }
    if (oldMode == 'k') {
        let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
        newMin = xMinInput.value * lambda / (1000 * 2 * Math.PI);
        newMax = xMaxInput.value * lambda / (1000 * 2 * Math.PI);
        if (newMode == 'angle') {
            newMin = Math.atan(newMin) / Math.PI * 180;
            newMax = Math.atan(newMax) / Math.PI * 180;
        }
    }
    X_RANGE_BUFFER = [newMin, newMax];
    axesChanged();
})

watch(yMinIndex, newIndex => {
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 1,
        startValue: newIndex,
    });
    chart1.dispatchAction({
        type: 'dataZoom',
        dataZoomIndex: 3,
        startValue: yMinInput.value,
    });
    if (prop.data!.wavelength) {
        chart1.dispatchAction({
            type: 'dataZoom',
            dataZoomIndex: 4,
            endValue: eVMode.value ? yMaxInput.value : CONST_1240 / yMinInput.value,
        });
    }
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
        endValue: yMaxInput.value,
    });
    if (prop.data!.wavelength) {
        chart1.dispatchAction({
            type: 'dataZoom',
            dataZoomIndex: 4,
            startValue: eVMode.value ? yMinInput.value : CONST_1240 / yMaxInput.value,
        });
    }
})

function axesChanged() {
    if (bindingLock.value) {
        emit('stretch', eVMode.value, xMode.value, tanMin.value, tanMax.value);
    } else {
        emit('stretch', eVMode.value, xMode.value, 0, prop.data!.height - 1);
    }
}

watch(eVMode, newVal => {
    EV_MODE = newVal;
    axesChanged();
})

function resetYRange() {
    if (dataSnapshot.value!.wavelength) {
        let minLambda = dataSnapshot.value!.wavelength[0];
        let maxLambda = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
        yMinInput.value = eVMode.value ? CONST_1240 / maxLambda : minLambda;
        yMaxInput.value = eVMode.value ? CONST_1240 / minLambda : maxLambda;
    } else {
        yMinInput.value = 0;
        yMaxInput.value = prop.data!.width - 1;
    }
}
function resetXIndex() {
    xMinIndex.value = 0;
    xMaxIndex.value = prop.data!.height - 1;
}

function resetNABinding() {
    inputNA.value = '';
}

function resetXRange() {
    let tan = Math.tan(Math.asin(inputNA.value));
    if (xMode.value == 'tan') {
        xMinInput.value = -tan;
        xMaxInput.value = tan;
    } 
    if (xMode.value == 'angle') {
        xMinInput.value = -Math.atan(tan) / Math.PI * 180;
        xMaxInput.value = Math.atan(tan) / Math.PI * 180;
    }
    if (xMode.value == 'k') {
        let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
        xMinInput.value = 2 * Math.PI * -tan / lambda * 1000;
        xMaxInput.value = 2 * Math.PI * tan / lambda * 1000;
    }
}

function copyToClipboard() {
    if (!prop.data) { return }

    emit('show-message', '数据已复制到剪贴板，可在Origin直接粘贴表格', 'ok');
    let str = '';
    if (prop.data!.wavelength) {
        str = `${inputNA.value ? 'tan(θ)' : 'Index'}\t${eVMode.value ? 'Energy' : 'Wavelength'}\tcounts
                \t${eVMode.value ? 'eV' : 'nm'}\n\n`;
    } else {
        str = `${inputNA.value ? 'tan(θ)' : 'Index'}\t\tcounts\n\n\n`;
    }

    if (inputNA.value) {
        let tan = Math.tan(Math.asin(inputNA.value));
        let height = xMaxIndex.value - xMinIndex.value + 1;
        let d = 2 * tan / (height - 1);
        for (var a = 0; a < height; a++) {
            str += `\t${-tan + a * d}`;
        }
    } else {
        for (var b = xMinIndex.value; b <= xMaxIndex.value; b++) {
            str += `\t${b}`;
        }
    }
    str += '\n';
    let yMin = yMinInput.value;
    let yMax = yMaxInput.value;
    if (prop.data!.wavelength) {
        let min = prop.data!.wavelength[0];
        let max = prop.data!.wavelength[prop.data!.width - 1];
        yMin = Math.round((yMin - min) / (max - min) * (prop.data!.width - 1));
        yMax = Math.round((yMax - min) / (max - min) * (prop.data!.width - 1));
    }
    for (var i = yMin; i <= yMax; i++) {
        if (prop.data!.wavelength) {
            str += `${eVMode.value ? (CONST_1240 / prop.data!.wavelength[i]) : prop.data!.wavelength[i]}`;
        }
        for (var j = xMinIndex.value; j <= xMaxIndex.value; j++) {
            str += `\t${prop.data!.frame[frameIndex.value][j * prop.data!.width + i]}`;
        }
        str += '\n';
    }
    writeText(str);
}
</script>

<template>
    <div id="arspc-container" @mouseover="focusing = true" @mouseleave="focusing = false"
        :class="['container', focusing ? 'container-focus' : '']">
        <p style="grid-column: 1 / -1; overflow: hidden;">{{ prop.path }}</p>
        <div id="ar-spectral-box">
            <div v-show="!prop.data" id="heatmap-placeholder">
                <h3>↖<br>&nbsp;&nbsp;&nbsp;&nbsp;点击
                    <IconOpen style="width: 5mm;" /> 打开文件，或将文件拖拽到窗口内
                </h3>
            </div>
            <div v-show="prop.data" ref="arspc" id="ar-spectral"></div>
        </div>
        <Transition name="pagination">
            <Pagination v-if="prop.data && prop.data.frame.length > 1" :length="prop.data.frame.length"
                v-model="frameIndex" style="grid-column-start: 1;" />
        </Transition>
        <button style="grid-column-start: 2;" @click="copyToClipboard" title="复制数据到剪贴板">
            <IconCopy />
        </button>
        <button id="save-ar-spec" style="grid-column-start: 3;" @click="saveImage(chart1, `角分辨光谱-${prop.name}`, !prop.data, silentPath).then(msg => {
            if (msg) { emit('show-message', msg, 'ok') }
        })" title="导出图片">
            <IconExport />
        </button>
    </div>
    <div id="arspc-options" @mouseover="focusing = true" @mouseleave="focusing = false"
        :class="['container', focusing ? 'container-focus' : '']">
        <div class="range-input-y">
            <label for="y-min-input">纵轴范围</label>
            <button @click="resetYRange" title="重置">
                <IconReset />
            </button>
            <input id="y-min-input" type="text" v-model.number="yMinInput">
            <label for="y-max-input">~</label>
            <input id="y-max-input" type="text" v-model.number="yMaxInput">
            <select v-if="prop.data?.wavelength" v-model="eVMode">
                <option :value="false">nm</option>
                <option :value="true">eV</option>
            </select>
        </div>
        <div class="x-binding-input">
            <label for="x-min-index">横轴索引</label>
            <button @click="resetXIndex" title="重置">
                <IconReset />
            </button>
            <input id="x-min-index" type="number" v-model.number="xMinIndex" :disabled="bindingLock">
            <label for="x-max-index">~</label>
            <input id="x-max-index" type="number" v-model.number="xMaxIndex" :disabled="bindingLock">
            <label for="NA-input" style="grid-column-start: 1">NA</label>
            <button @click="resetNABinding" title="重置">
                <IconReset />
            </button>
            <input id="NA-input" type="text" v-model.number="inputNA" :disabled="bindingLock">
            <button v-show="inputNA" :class="bindingLock ? 'button-active' : ''" style="grid-column-start: 5;"
                @click="bindingLock = !bindingLock" :title="bindingLock ? '解绑' : '绑定'">
                <IconLock v-show="bindingLock" />
                <IconUnlock v-show="!bindingLock" />
                ⤴️
            </button>
        </div>
        <div v-if="bindingLock" class="range-input-x">
            <label for="x-min-input">横轴范围</label>
            <button @click="resetXRange" title="重置">
                <IconReset />
            </button>
            <input id="x-min-input" type="text" v-model.number="xMinInput">
            <label for="x-max-input">~</label>
            <input id="x-max-input" type="text" v-model.number="xMaxInput">
            <select v-model="xMode">
                <option value="tan">tan(θ)</option>
                <option value="angle">°</option>
                <option v-if="prop.data?.wavelength" value="k">μm^-1(k)</option>
            </select>
        </div>
    </div>
</template>

<style>
#arspc-container {
    grid-row: 1 / 3;
    grid-template-columns: 1fr auto auto;
    grid-template-rows: auto 1fr auto;
}

#ar-spectral-box {
    background-color: white;
    max-width: 12cm;
    max-height: 16cm;
    border-radius: 1mm;
    grid-row: 2 / 3;
    grid-column: 1 / -1;
    align-self: center;
    overflow: auto;
}

.pagination-enter-active,
.pagination-leave-active {
  transition: all 0.3s ease;
}

.pagination-enter-from,
.pagination-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

#heatmap-placeholder,
#ar-spectral,
#ar-spectral-hi-res {
    background-color: white;
    width: 12cm;
    height: 16cm;
}

#arspc-options {
    padding: 2mm 4mm;
    justify-content: left;
    align-content: center;
    gap: 1mm;
    overflow-y: auto;
    overflow-x: hidden;
}

.range-input-y,
.range-input-x {
    display: flex;
    gap: 1mm;
    justify-content: left;
    align-items: center;
}

.x-binding-input {
    display: grid;
    grid-template-columns: repeat(5, auto);
    grid-row: repeat(2, auto);
    gap: 1mm;
    justify-content: left;
    align-items: center;
}

.x-binding-input p {
    user-select: none;
    -webkit-user-select: none;
    cursor: default;
}
</style>