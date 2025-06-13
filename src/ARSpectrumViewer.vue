<script setup lang="ts">
import { computed, inject, nextTick, onMounted, Ref, ref, useTemplateRef, watch } from 'vue';
import * as echarts from 'echarts';
import { downSampling, drawARSpec, SpeData, saveImage, CONST_1240, compatible } from './scripts/DataViewer';
import Pagination from './components/Pagination.vue';
import ModeSwitch from './components/ModeSwitch.vue';
import IconCopy from './assets/clipboard.svg?component';
import IconExport from './assets/down-picture.svg?component';
import IconOpen from './assets/folder-open.svg?component';
import IconReset from './assets/undo.svg?component';
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

const emit = defineEmits(['show-message', 'slice-at-index']);

const xMinIndex = ref();
const xMaxIndex = ref();

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
        if (dataSnapshot.value.wavelength) {
            let min = dataSnapshot.value.wavelength[0];
            let max = dataSnapshot.value.wavelength[dataSnapshot.value!.width - 1];
            return Math.round((yMinLambda.value - min) / (max - min) * (dataSnapshot.value.width - 1));
        } else {
            return Math.round(yMinLambda.value / (prop.data!.width - 1) * (dataSnapshot.value.width - 1));
        }
    }
});
const yMaxIndex = computed(() => {
    if (dataSnapshot.value) {
        if (dataSnapshot.value.wavelength) {
            let min = dataSnapshot.value.wavelength[0];
            let max = dataSnapshot.value.wavelength[dataSnapshot.value!.width - 1];
            return Math.round((yMaxLambda.value - min) / (max - min) * (dataSnapshot.value.width - 1));
        } else {
            return Math.round(yMaxLambda.value / (prop.data!.width - 1) * (dataSnapshot.value.width - 1));
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
                    yMin: 0, yMax: prop.data!.width - 1,
                    xMin: 0, xMax: prop.data!.height - 1
                });
                yAxisMode.value = true;
                xMinIndex.value = 0;
                xMaxIndex.value = newData!.height - 1;
                yMinLambda.value = newData!.wavelength ? newData!.wavelength[0] : 0;
                yMaxLambda.value = newData!.wavelength ? newData!.wavelength[newData!.width - 1] : newData!.width - 1;
                bindingNA.value = '';
            } else {
                let info = compatible(newData, oldData, xMinIndex.value, xMaxIndex.value, yMinLambda.value, yMaxLambda.value);
                drawARSpec(chart1, dataSnapshot.value!, {
                    name: prop.name, frameIndex: frameIndex.value,
                    ...info
                });
                if (!(newData.wavelength && oldData.wavelength)) {
                    yAxisMode.value = true;
                }
                xMinIndex.value = info.xMin;
                xMaxIndex.value = info.xMax;
                yMinLambda.value = info.yMin;
                yMaxLambda.value = info.yMax;
            }
        }
    })
}, { immediate: true });

watch(frameIndex, newIndex => {
    drawARSpec(chart1, dataSnapshot.value!, {
        name: prop.name, frameIndex: newIndex,
        yMin: 0, yMax: prop.data!.width - 1,
        xMin: 0, xMax: prop.data!.height - 1
    });
})

const bindingNA = ref();

watch(bindingNA, newVal => {
    let inputed = newVal && newVal != '';
    let xAxisOptions = (chart1.getOption().xAxis as { [key: string]: any }[]);
    xAxisOptions[1].show = !inputed;
    xAxisOptions[2].show = inputed;
    if (inputed) {
        let tan = Math.tan(Math.asin(newVal));
        xAxisOptions[2].min = -tan;
        xAxisOptions[2].max = tan;
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
    if (prop.data!.wavelength) {
        chart1.dispatchAction({
            type: 'dataZoom',
            dataZoomIndex: 4,
            startValue: yMaxEnergy.value,
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
        endValue: yMaxLambda.value,
    });
    if (prop.data!.wavelength) {
        chart1.dispatchAction({
            type: 'dataZoom',
            dataZoomIndex: 4,
            endValue: yMinEnergy.value,
        });
    }
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

function resetYRange() {
    if (dataSnapshot.value!.wavelength) {
        yMinLambda.value = dataSnapshot.value!.wavelength[0];
        yMaxLambda.value = dataSnapshot.value!.wavelength[dataSnapshot.value!.width - 1];
    } else {
        yMinLambda.value = 0;
        yMaxLambda.value = prop.data!.width - 1;
    }
}
function resetXRange() {
    xMinIndex.value = 0;
    xMaxIndex.value = prop.data!.height - 1;
}
function resetNABinding() {
    bindingNA.value = '';
}

function copyToClipboard() {
    if (!prop.data) { return }

    emit('show-message', '数据已复制到剪贴板，可在Origin直接粘贴表格', 'ok');
    let str = '';
    if (prop.data!.wavelength) {
        str = `${bindingNA.value ? 'tan(θ)' : 'Index'}\t${yAxisMode.value ? 'Wavelength' : 'Energy'}\tcounts
                \t${yAxisMode.value ? 'nm' : 'eV'}\n\n`;
    } else {
        str = `${bindingNA.value ? 'tan(θ)' : 'Index'}\t\tcounts\n\n\n`;
    }

    if (bindingNA.value) {
        let tan = Math.tan(Math.asin(bindingNA.value));
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
    let yMin = yMinLambda.value;
    let yMax = yMaxLambda.value;
    if (prop.data!.wavelength) {
        let min = prop.data!.wavelength[0];
        let max = prop.data!.wavelength[prop.data!.width - 1];
        yMin = Math.round((yMin - min) / (max - min) * (prop.data!.width - 1));
        yMax = Math.round((yMax - min) / (max - min) * (prop.data!.width - 1));
    }
    for (var i = yMin; i <= yMax; i++) {
        if (prop.data!.wavelength) {
            str += `${yAxisMode.value ? prop.data!.wavelength[i] : (CONST_1240 / prop.data!.wavelength[i])}`;
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
        <button id="save-ar-spec" style="grid-column-start: 3;"
        @click="saveImage(chart1, `角分辨光谱-${prop.name}`, !prop.data, silentPath).then(msg => {
            if (msg) { emit('show-message', msg, 'ok') }
        })" title="导出图片">
            <IconExport />
        </button>
    </div>
    <div id="arspc-options" @mouseover="focusing = true" @mouseleave="focusing = false"
        :class="['container', focusing ? 'container-focus' : '']">
        <ModeSwitch v-if="prop.data?.wavelength" :name="'纵轴模式'" :mode1="'波长'" :mode2="'能量'" v-model="yAxisMode" />
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
            <label for="binding-na" style="grid-column-start: 1">NA 绑定</label>
            <button @click="resetNABinding" title="重置">
                <IconReset />
            </button>
            <input id="binding-na" type="text" v-model.number="bindingNA">
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