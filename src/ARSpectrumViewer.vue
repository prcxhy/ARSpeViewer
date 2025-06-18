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
    newlyOpen: boolean
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
var X_MIN_BUFFER = [0, 0, 0];
var X_MAX_BUFFER = [0, 0, 0];
var X_INDEX_RANGE = [0, 0];
var X_SNAPSHOT_INDEX_RANGE = [0, 0];
const xMinInput = computed({
    get: () => {
        switch (xMode.value) {
            case 'tan': return X_MIN_BUFFER[0];
            case 'angle': return X_MIN_BUFFER[1];
            case 'k': return X_MIN_BUFFER[2];
        }
    },
    set: (val: number) => {
        let min = 0;
        switch (xMode.value) {
            case 'tan': X_MIN_BUFFER[0] = val;
                X_MIN_BUFFER[1] = Math.atan(val) / Math.PI * 180;
                if (dataSnapshot.value!.wavelength) {
                    let lambda = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
                    X_MIN_BUFFER[2] = 2 * Math.PI * val / lambda * 1000;
                }
                min = Math.round((val - tanMin.value) / (tanMax.value - tanMin.value) * (dataSnapshot.value!.height - 1));
                break;
            case 'angle': X_MIN_BUFFER[1] = val;
                X_MIN_BUFFER[0] = Math.tan(val * Math.PI / 180);
                if (dataSnapshot.value!.wavelength) {
                    let lambda = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
                    X_MIN_BUFFER[2] = 2 * Math.PI * X_MIN_BUFFER[0] / lambda * 1000;
                }
                let minAngle = Math.atan(tanMin.value) / Math.PI * 180;
                let maxAngle = Math.atan(tanMax.value) / Math.PI * 180;
                min = Math.round((val - minAngle) / (maxAngle - minAngle) * (dataSnapshot.value!.height - 1));
                break;
            case 'k': X_MIN_BUFFER[2] = val;
                let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
                X_MIN_BUFFER[0] = val * lambda / (1000 * 2 * Math.PI);
                X_MIN_BUFFER[1] = Math.atan(X_MIN_BUFFER[0]) / Math.PI * 180;
                let minK = 2 * Math.PI * tanMin.value / lambda * 1000;
                let maxK = 2 * Math.PI * tanMax.value / lambda * 1000;
                min = Math.round((val - minK) / (maxK - minK) * (dataSnapshot.value!.height - 1));
                break;
        }
        chart1.dispatchAction({
            type: 'dataZoom',
            batch: [{
                dataZoomIndex: 0,
                startValue: min,
            }, {
                dataZoomIndex: 5,
                startValue: val,
            }]
        });
        X_INDEX_RANGE[0] = min;
    }
});

const xMaxInput = computed({
    get: () => {switch (xMode.value) {
            case 'tan': return X_MAX_BUFFER[0];
            case 'angle': return X_MAX_BUFFER[1];
            case 'k': return X_MAX_BUFFER[2];
        }
    },
    set: (val: number) => {
        let max = 0;
        switch (xMode.value) {
            case 'tan': X_MAX_BUFFER[0] = val;
                X_MAX_BUFFER[1] = Math.atan(val) / Math.PI * 180;
                if (dataSnapshot.value!.wavelength) {
                    let lambda = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
                    X_MAX_BUFFER[2] = 2 * Math.PI * val / lambda * 1000;
                }
                max = Math.round((val - tanMin.value) / (tanMax.value - tanMin.value) * (dataSnapshot.value!.height - 1));
                break;
            case 'angle': X_MAX_BUFFER[1] = val;
                X_MAX_BUFFER[0] = Math.tan(val * Math.PI / 180);
                if (dataSnapshot.value!.wavelength) {
                    let lambda = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
                    X_MAX_BUFFER[2] = 2 * Math.PI * X_MAX_BUFFER[0] / lambda * 1000;
                }
                let minAngle = Math.atan(tanMin.value) / Math.PI * 180;
                let maxAngle = Math.atan(tanMax.value) / Math.PI * 180;
                max = Math.round((val - minAngle) / (maxAngle - minAngle) * (dataSnapshot.value!.height - 1));
                break;
            case 'k': X_MAX_BUFFER[2] = val;
                let lambda = dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
                X_MAX_BUFFER[0] = val * lambda / (1000 * 2 * Math.PI);
                X_MAX_BUFFER[1] = Math.atan(X_MAX_BUFFER[0]) / Math.PI * 180;
                let minK = 2 * Math.PI * tanMin.value / lambda * 1000;
                let maxK = 2 * Math.PI * tanMax.value / lambda * 1000;
                max = Math.round((val - minK) / (maxK - minK) * (dataSnapshot.value!.height - 1));
                break;
        }
        chart1.dispatchAction({
            type: 'dataZoom',
            batch: [{
                dataZoomIndex: 0,
                endValue: max,
            }, {
                dataZoomIndex: 5,
                endValue: val,
            }]
        });
        X_INDEX_RANGE[1] = max;
    }
});

const eVMode = ref(false);
var EV_MODE = false;
var NM_BUFFER = [0, 0];
var EV_BUFFER = [0, 0];
const yMinInput = computed({
    get: () => { if (prop.data) { return eVMode.value ? EV_BUFFER[0] : NM_BUFFER[0]}},
    set: (val: number) => {
        let dzi = 3;
        if (eVMode.value) {
            EV_BUFFER[0] = val;
            NM_BUFFER[1] = CONST_1240 / val;
            let max = CONST_1240 / dataSnapshot.value!.wavelength![0];
            let min = CONST_1240 / dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
            chart1.dispatchAction({
                type: 'dataZoom',
                dataZoomIndex: 1,
                endValue: Math.round((max - val) / (max - min) * (dataSnapshot.value!.width - 1)),
            });
            Y_INDEX_RANGE[1] = Math.round((max - val) / (max - min) * (prop.data!.width - 1));
            dzi = 4;
        } else {
            NM_BUFFER[0] = val;
            if (dataSnapshot.value!.wavelength) {
                EV_BUFFER[1] = CONST_1240 / val;
                let min = dataSnapshot.value!.wavelength[0];
                let max = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
                chart1.dispatchAction({
                    type: 'dataZoom',
                    dataZoomIndex: 1,
                    startValue: Math.round((val - min) / (max - min) * (dataSnapshot.value!.width - 1)),
                });
                Y_INDEX_RANGE[0] = Math.round((val - min) / (max - min) * (prop.data!.width - 1));
            } else {
                chart1.dispatchAction({
                    type: 'dataZoom',
                    dataZoomIndex: 1,
                    startValue: Math.round(val / (prop.data!.width - 1) * (dataSnapshot.value!.width - 1)),
                });
                Y_INDEX_RANGE[0] = val;
            }
            
        }
        chart1.dispatchAction({
            type: 'dataZoom',
            dataZoomIndex: dzi,
            startValue: val,
        });
    }
});

const yMaxInput = computed({
    get: () => { if (prop.data) { return eVMode.value ? EV_BUFFER[1] : NM_BUFFER[1] } },
    set: (val: number) => {
        let dzi = 3;
        if (eVMode.value) {
            EV_BUFFER[1] = val;
            NM_BUFFER[0] = CONST_1240 / val;
            let max = CONST_1240 / dataSnapshot.value!.wavelength![0];
            let min = CONST_1240 / dataSnapshot.value!.wavelength![dataSnapshot.value!.width - 1];
            chart1.dispatchAction({
                type: 'dataZoom',
                dataZoomIndex: 1,
                startValue: Math.round((max - val) / (max - min) * (dataSnapshot.value!.width - 1)),
            });
            Y_INDEX_RANGE[0] = Math.round((max - val) / (max - min) * (prop.data!.width - 1));
            dzi = 4;
        } else {
            NM_BUFFER[1] = val;
            if (dataSnapshot.value!.wavelength) {
                EV_BUFFER[0] = CONST_1240 / val;
                let min = dataSnapshot.value!.wavelength[0];
                let max = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
                chart1.dispatchAction({
                    type: 'dataZoom',
                    dataZoomIndex: 1,
                    endValue: Math.round((val - min) / (max - min) * (dataSnapshot.value!.width - 1)),
                });
                Y_INDEX_RANGE[1] = Math.round((val - min) / (max - min) * (prop.data!.width - 1));
            } else {
                chart1.dispatchAction({
                    type: 'dataZoom',
                    dataZoomIndex: 1,
                    endValue: Math.round(val / (prop.data!.width - 1) * (dataSnapshot.value!.width - 1)),
                });
                Y_INDEX_RANGE[1] = val;
            }
            
        }
        chart1.dispatchAction({
            type: 'dataZoom',
            dataZoomIndex: dzi,
            endValue: val,
        });
    }
});

var Y_INDEX_RANGE = [0, 0];

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
    if (newData && newData.wavelength) {
        NM_BUFFER = [newData!.wavelength[0], newData!.wavelength[newData!.width - 1]];
        EV_BUFFER = [CONST_1240 / NM_BUFFER[1], CONST_1240 / NM_BUFFER[0]];
    } else if (newData) {
        NM_BUFFER = [0, newData!.width - 1];
    }
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
                inputNA.value = '';
            } else if (!prop.newlyOpen) {
                drawARSpec(chart1, dataSnapshot.value!, {
                    name: prop.name, frameIndex: frameIndex.value,
                    eVMode: eVMode.value, xMode: xMode.value,
                    yMin: 0, yMax: prop.data!.width - 1,
                    xMinIndex: 0, xMaxIndex: prop.data!.height - 1,
                    xMin: tanMin.value, xMax: tanMax.value,
                });
                if (bindingLock.value) {
                    chart1.dispatchAction({
                        type: 'dataZoom',
                        dataZoomIndex: 5,
                        startValue: xMinInput.value,
                        endValue: xMaxInput.value,
                    })
                } else {
                    chart1.dispatchAction({
                        type: 'dataZoom',
                        dataZoomIndex: 0,
                        startValue: X_SNAPSHOT_INDEX_RANGE[0],
                        endValue: X_SNAPSHOT_INDEX_RANGE[1],
                    });
                }
            } else {
                let info = compatible(newData, oldData, xMinIndex.value, xMaxIndex.value, yMinInput.value!, yMaxInput.value!);
                if (!(newData.wavelength && oldData.wavelength)) {
                    eVMode.value = false;
                }
                yMinInput.value = eVMode.value ? CONST_1240 / info.yMax : info.yMin;
                yMaxInput.value = eVMode.value ? CONST_1240 / info.yMin : info.yMax;
                if (!info.xCompate) {
                    xMinIndex.value = info.xMinIndex;
                    xMaxIndex.value = info.xMaxIndex;
                    bindingLock.value = false;
                }
                if (!info.yCompate && info.xCompate && xMode.value == 'k') {
                    xMode.value = 'tan';
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
const tanMin = computed(() => {
    let tan = Math.tan(Math.asin(inputNA.value));
    let a = xMaxIndex.value - xMinIndex.value;
    return -tan - xMinIndex.value / a * 2 * tan;
});
const tanMax = computed(() => {
    let tan = Math.tan(Math.asin(inputNA.value));
    let a = xMaxIndex.value - xMinIndex.value;
    return tan + (prop.data!.height - 1 - xMaxIndex.value) / a * 2 * tan;
});

watch(bindingLock, locked => {
    let tan = Math.tan(Math.asin(inputNA.value));
    if (locked) {
        xMinInput.value = -tan;
        xMaxInput.value = tan;
    } else {
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
    X_INDEX_RANGE[0] = newIndex;
    let min = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value!.height - 1));
    X_SNAPSHOT_INDEX_RANGE[0] = min;
    chart1.dispatchAction({
        type: 'dataZoom',
        batch: [{
            dataZoomIndex: 0,
            startValue: min,
        }, {
            dataZoomIndex: 2,
            startValue: newIndex,
        }]
    });
});

watch(xMaxIndex, newIndex => {
    X_INDEX_RANGE[1] = newIndex;
    let max = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value!.height - 1));
    X_SNAPSHOT_INDEX_RANGE[1] = max;
    chart1.dispatchAction({
        type: 'dataZoom',
        batch: [{
            dataZoomIndex: 0,
            endValue: max,
        }, {
            dataZoomIndex: 2,
            endValue: newIndex,
        }]
    });
});

watch(xMode, axesChanged)

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
    let xName = 'Index';
    let xUnit = '';
    if (bindingLock.value) {
        switch (xMode.value) {
            case 'tan': xName = 'tan(θ)';
                break;
            case 'angle': xName = 'θ';
                xUnit = '°'
                break;
            case 'k': xName = 'k';
                xUnit = 'μm⁻¹'
                break;
        }
    }
    let yName = '';
    let yUnit = '';
    if (prop.data.wavelength) {
        yName = EV_MODE ? 'Energy' : 'Wavelength';
        yUnit = EV_MODE ? 'eV' : 'nm';
    }
    str = `${ xName }\t${ yName }\tcounts\n${ xUnit }\t${ yUnit }\n\n`;

    if (bindingLock.value) {
        let height = X_INDEX_RANGE[1] - X_INDEX_RANGE[0];
        let d = (xMaxInput.value! - xMinInput.value!) / (height - 1);
        for (var a = 0; a <= height; a++) {
            str += `\t${xMinInput.value! + a * d}`;
        }
    } else {
        for (var b = X_INDEX_RANGE[0]; b <= X_INDEX_RANGE[1]; b++) {
            str += `\t${b}`;
        }
    }
    str += '\n';
    
    for (var i = Y_INDEX_RANGE[0]; i <= Y_INDEX_RANGE[1]; i++) {
        if (prop.data!.wavelength) {
            str += `${eVMode.value ? (CONST_1240 / prop.data!.wavelength[i]) : prop.data!.wavelength[i]}`;
        }
        for (var j = X_INDEX_RANGE[0]; j <= X_INDEX_RANGE[1]; j++) {
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
        <Transition name="fade">
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
            <button @click="resetXIndex" title="重置" :disabled="bindingLock">
                <IconReset />
            </button>
            <input id="x-min-index" type="number" v-model.number="xMinIndex" :disabled="bindingLock">
            <label for="x-max-index">~</label>
            <input id="x-max-index" type="number" v-model.number="xMaxIndex" :disabled="bindingLock">
            <label for="NA-input" style="grid-column-start: 1">NA</label>
            <button @click="resetNABinding" title="重置" :disabled="bindingLock">
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
        <Transition name="fade">
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
                    <option value="angle">° (角度)</option>
                    <option v-if="prop.data?.wavelength" value="k">μm⁻¹ (k)</option>
                </select>
            </div>
        </Transition>
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

.fade-enter-active,
.fade-leave-active {
  transition: all 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
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