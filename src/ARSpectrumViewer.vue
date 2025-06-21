<script setup lang="ts">
import { computed, inject, nextTick, onMounted, reactive, Ref, ref, useTemplateRef, watch } from 'vue';
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
import { XRange, YRange } from './scripts/ParametersConvert';

const prop = defineProps<{
    data?: SpeData
    name: string
    path: string
    newlyOpen: boolean
    hightlightIndex: number
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
const inputNA = ref();
var GLOBAL_RANGE_X = computed(() => {
    return new XRange(
        inputNA.value,
        prop.data!.wavelength ? prop.data!.wavelength[prop.data!.width - 1] : prop.data!.width - 1,
        [xMinIndex.value, xMaxIndex.value, prop.data!.height - 1]);
});
var INPUT_RANGE_X = reactive<XRange>(new XRange(1, 1000));
const xMinInput = computed({
    get: () => {
        switch (xMode.value) {
            case 'tan': return INPUT_RANGE_X.minTan;
            case 'angle': return INPUT_RANGE_X.minAngle;
            case 'k': return INPUT_RANGE_X.minK;
        }
    },
    set: (val: number) => {
        let indexRange: number[] = [];
        switch (xMode.value) {
            case 'tan': INPUT_RANGE_X.minTan = val;
                indexRange = INPUT_RANGE_X.indexRangeOfTanIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                break;
            case 'angle': INPUT_RANGE_X.minAngle = val;
                indexRange = INPUT_RANGE_X.indexRangeOfAngleIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                break;
            case 'k': INPUT_RANGE_X.minK = val;
                indexRange = INPUT_RANGE_X.indexRangeOfKIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                break;
        }

        chart1.dispatchAction({
            type: 'dataZoom',
            batch: [{
                dataZoomIndex: 0,
                startValue: indexRange[0],
            }, {
                dataZoomIndex: 5,
                startValue: val,
            }]
        });
    }
});

const xMaxInput = computed({
    get: () => {
        switch (xMode.value) {
            case 'tan': return INPUT_RANGE_X.maxTan;
            case 'angle': return INPUT_RANGE_X.maxAngle;
            case 'k': return INPUT_RANGE_X.maxK;
        }
    },
    set: (val: number) => {
        let indexRange: number[] = [];
        switch (xMode.value) {
            case 'tan': INPUT_RANGE_X.maxTan = val;
                indexRange = INPUT_RANGE_X.indexRangeOfTanIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                break;
            case 'angle': INPUT_RANGE_X.maxAngle = val;
                indexRange = INPUT_RANGE_X.indexRangeOfAngleIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                break;
            case 'k': INPUT_RANGE_X.maxK = val;
                indexRange = INPUT_RANGE_X.indexRangeOfKIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                break;
        }

        chart1.dispatchAction({
            type: 'dataZoom',
            batch: [{
                dataZoomIndex: 0,
                endValue: indexRange[1],
            }, {
                dataZoomIndex: 5,
                endValue: val,
            }]
        });
    }
});

const eVMode = ref(false);
var EV_MODE = false;
var INPUT_RANGE_Y = reactive<YRange>(new YRange([400, 1000]));
var GLOBAL_RANGE_Y: YRange;
const yMinInput = computed({
    get: () => { if (prop.data) { return eVMode.value ? INPUT_RANGE_Y.minEnergy : INPUT_RANGE_Y.minLambda}},
    set: (val: number) => {
        let dzi = 3;
        let setValue = {};
        if (eVMode.value) {
            INPUT_RANGE_Y.minEnergy = val;

            let indexRange = INPUT_RANGE_Y.indexRangeOfEnergyIn(GLOBAL_RANGE_Y, dataSnapshot.value!.width - 1)
            setValue = { endValue: indexRange[1] };

            dzi = 4;
        } else {
            INPUT_RANGE_Y.minLambda = val;
            if (dataSnapshot.value!.wavelength) {
                let indexRange = INPUT_RANGE_Y.indexRangeOfLambdaIn(GLOBAL_RANGE_Y, dataSnapshot.value!.width - 1)
                setValue = { startValue: indexRange[0] };
            } else {
                setValue = { startValue: Math.round(val / (prop.data!.width - 1) * (dataSnapshot.value!.width - 1)) };
            }
        }
        chart1.dispatchAction({
            type: 'dataZoom',
            batch: [{
                dataZoomIndex: 1,
                ...setValue
            }, {
                dataZoomIndex: dzi,
                startValue: val,
            }]
        });
    }
});

const yMaxInput = computed({
    get: () => { if (prop.data) { return eVMode.value ? INPUT_RANGE_Y.maxEnergy : INPUT_RANGE_Y.maxLambda}},
    set: (val: number) => {
        let dzi = 3;
        let setValue = {};
        if (eVMode.value) {
            INPUT_RANGE_Y.maxEnergy = val;

            let indexRange = INPUT_RANGE_Y.indexRangeOfEnergyIn(GLOBAL_RANGE_Y, dataSnapshot.value!.width - 1)
            setValue = { startValue: indexRange[0] };

            dzi = 4;
        } else {
            INPUT_RANGE_Y.maxLambda = val;
            if (dataSnapshot.value!.wavelength) {
                let indexRange = INPUT_RANGE_Y.indexRangeOfLambdaIn(GLOBAL_RANGE_Y, dataSnapshot.value!.width - 1)
                setValue = { endValue: indexRange[1] };
            } else {
                setValue = { endValue: Math.round(val / (prop.data!.width - 1) * (dataSnapshot.value!.width - 1)) };
            }
        }
        chart1.dispatchAction({
            type: 'dataZoom',
            batch: [{
                dataZoomIndex: 1,
                ...setValue
            }, {
                dataZoomIndex: dzi,
                endValue: val,
            }]
        });
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
    if (newData) {
        let wavelength = newData.wavelength ? newData.wavelength : [0, newData.width - 1]
        GLOBAL_RANGE_Y = new YRange(wavelength);
        if (!oldData) {
            INPUT_RANGE_Y = reactive(new YRange(wavelength));
        }
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
                    xMin: GLOBAL_RANGE_X.value.minTan, xMax: GLOBAL_RANGE_X.value.maxTan,
                });
                let indexRangeY = EV_MODE ?
                INPUT_RANGE_Y.indexRangeOfEnergyIn(GLOBAL_RANGE_Y, dataSnapshot.value!.width - 1) :
                INPUT_RANGE_Y.indexRangeOfLambdaIn(GLOBAL_RANGE_Y, dataSnapshot.value!.width - 1)
                let dataZoomBatch = [{
                    dataZoomIndex: 1,
                    startValue: indexRangeY[0],
                    endValue: indexRangeY[1]
                }, {
                    dataZoomIndex: EV_MODE ? 4 : 3,
                    startValue: yMinInput.value,
                    endValue: yMaxInput.value,
                }];
                if (bindingLock.value) {
                    let indexRangeX: number[] = [];
                    switch (xMode.value) {
                        case 'tan': indexRangeX = INPUT_RANGE_X.indexRangeOfTanIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                            break;
                        case 'angle': indexRangeX = INPUT_RANGE_X.indexRangeOfAngleIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                            break;
                        case 'k': indexRangeX = INPUT_RANGE_X.indexRangeOfKIn(GLOBAL_RANGE_X.value, dataSnapshot.value!.height - 1)
                            break;
                    }
                    dataZoomBatch.push({
                        dataZoomIndex: 0,
                        startValue: indexRangeX[0],
                        endValue: indexRangeX[1],
                    }, {
                        dataZoomIndex: 5,
                        startValue: xMinInput.value,
                        endValue: xMaxInput.value,
                    });
                } else {
                    dataZoomBatch.push({
                        dataZoomIndex: 0,
                        startValue: Math.round(xMinIndex.value * (dataSnapshot.value!.height - 1) / (prop.data!.height - 1)),
                        endValue: Math.round(xMaxIndex.value * (dataSnapshot.value!.height - 1) / (prop.data!.height - 1)),
                    });
                }
                chart1.dispatchAction({
                    type: 'dataZoom',
                    batch: dataZoomBatch
                });
            } else {
                AXES_CHANGE_LOCK = true;
                let info = compatible(newData, oldData, xMinIndex.value, xMaxIndex.value, yMinInput.value!, yMaxInput.value!);
                if (!(newData.wavelength && oldData.wavelength)) {
                    eVMode.value = false;
                }
                if (!info.xCompate) {
                    xMode.value = 'tan';
                    bindingLock.value = false;
                }
                if (!info.yCompate && info.xCompate && xMode.value == 'k') {
                    xMode.value = 'tan';
                }
                xMinIndex.value = info.xMinIndex;
                xMaxIndex.value = info.xMaxIndex;
                if (eVMode.value || xMode.value != 'tan') {
                    INPUT_RANGE_Y.minLambda = info.yMin;
                    INPUT_RANGE_Y.maxLambda = info.yMax;
                    axesChanged();
                } else {
                    drawARSpec(chart1, dataSnapshot.value!, {
                        name: prop.name, frameIndex: frameIndex.value,
                        eVMode: eVMode.value, xMode: xMode.value,
                        yMin: 0, yMax: prop.data!.width - 1,
                        xMinIndex: 0, xMaxIndex: prop.data!.height - 1,
                        xMin: GLOBAL_RANGE_X.value.minTan, xMax: GLOBAL_RANGE_X.value.maxTan
                    });
                    yMinInput.value = eVMode.value ? CONST_1240 / info.yMax : info.yMin;
                    yMaxInput.value = eVMode.value ? CONST_1240 / info.yMin : info.yMax;
                }
                AXES_CHANGE_LOCK = false;
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
        xMin: GLOBAL_RANGE_X.value.minTan, xMax: GLOBAL_RANGE_X.value.maxTan
    });
})

watch(() => prop.hightlightIndex, (newIndex, oldIndex) => {
    let height = dataSnapshot.value!.height;
    let snapshotIndexNew = Math.round(newIndex / (prop.data!.height - 1) * (height - 1));
    let snapshotIndexOld = Math.round(oldIndex / (prop.data!.height - 1) * (height - 1));
    let newRange = Array.from({ length: dataSnapshot.value!.width }, (_, i) => snapshotIndexNew + i * height);
    let oldRange = Array.from({ length: dataSnapshot.value!.width }, (_, i) => snapshotIndexOld + i * height);
    chart1!.dispatchAction({
        type: 'highlight',
        seriesIndex: 0,
        dataIndex: newRange
    })
    chart1!.dispatchAction({
        type: 'downplay',
        seriesIndex: 0,
        dataIndex: oldRange
    })
    setTimeout(() => chart1!.dispatchAction({
        type: 'downplay',
        seriesIndex: 0,
        dataIndex: newRange
    }), 1000);
})

watch(bindingLock, locked => {
    let tan = Math.tan(Math.asin(inputNA.value));
    if (locked) {
        let lambda = prop.data!.wavelength ? prop.data!.wavelength[prop.data!.width - 1] : prop.data!.width - 1;
        INPUT_RANGE_X = reactive(new XRange(inputNA.value, lambda))
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
    let min = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value!.height - 1));
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
    let max = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value!.height - 1));
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

var AXES_CHANGE_LOCK = false;

function axesChanged() {
    if (bindingLock.value) {
        emit('stretch', eVMode.value, xMode.value, GLOBAL_RANGE_X.value.minTan, GLOBAL_RANGE_X.value.maxTan);
    } else {
        emit('stretch', eVMode.value, xMode.value, 0, prop.data!.height - 1);
    }
}

watch(xMode, () => {
    if (!AXES_CHANGE_LOCK) {
        axesChanged()
    }
})

watch(eVMode, newVal => {
    EV_MODE = newVal;
    if (!AXES_CHANGE_LOCK) {
        axesChanged();
    }
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

    let indexRangeX: number[] = [];
    if (bindingLock.value) {
        switch (xMode.value) {
            case 'tan': indexRangeX = INPUT_RANGE_X.indexRangeOfTanIn(GLOBAL_RANGE_X.value, prop.data.height - 1)
                break;
            case 'angle': indexRangeX = INPUT_RANGE_X.indexRangeOfAngleIn(GLOBAL_RANGE_X.value, prop.data.height - 1)
                break;
            case 'k': indexRangeX = INPUT_RANGE_X.indexRangeOfKIn(GLOBAL_RANGE_X.value, prop.data.height - 1)
                break;
        }
        let height = indexRangeX[1] - indexRangeX[0];
        let d = (xMaxInput.value! - xMinInput.value!) / (height - 1);
        for (var a = 0; a <= height; a++) {
            str += `\t${xMinInput.value! + a * d}`;
        }
    } else {
        indexRangeX = [xMinIndex.value, xMaxIndex.value]
        for (var b = xMinIndex.value; b <= xMaxIndex.value; b++) {
            str += `\t${b}`;
        }
    }
    str += '\n';
    
    let indexRangeY = EV_MODE ?
    INPUT_RANGE_Y.indexRangeOfEnergyIn(GLOBAL_RANGE_Y, prop.data!.width - 1) :
    INPUT_RANGE_Y.indexRangeOfLambdaIn(GLOBAL_RANGE_Y, prop.data!.width - 1);
    for (var i = indexRangeY[0]; i <= indexRangeY[1]; i++) {
        if (prop.data!.wavelength) {
            str += `${EV_MODE ? (CONST_1240 / prop.data!.wavelength[i]) : prop.data!.wavelength[i]}`;
        }
        for (var j = indexRangeX[0]; j <= indexRangeX[1]; j++) {
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