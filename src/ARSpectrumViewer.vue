<script setup lang="ts">
import { computed, nextTick, ref, useTemplateRef, watch } from 'vue';
import * as echarts from 'echarts';
import { downSampling, drawARSpec, SpeData, saveImage } from './scripts/DataViewer';
import ModeSwitch from './components/ModeSwitch.vue';
import IconOption from './assets/config.svg?component';
import IconArrow from './assets/right.svg?component';
import IconCopy from './assets/clipboard.svg?component';
import IconExport from './assets/export.svg?component';
import IconReset from './assets/undo.svg?component';
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

const H = 6.62607015e-34;
const C0 = 299792458;
const e = 1.602176634e-19;
const CONST_1240 = H * C0 * 1e9 / e;

const prop = defineProps<{
    data?: SpeData
    name: string
}>()

var chart1: echarts.ECharts;
const arspc = useTemplateRef('arspc');

const dataSnapshot = computed(() => {
    return downSampling(prop.data!, 3);
})

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
        // console.log('min energy input: ' + val)
        yMaxLambda.value = CONST_1240 / val;
    }
});
const yMaxEnergy = computed({
    get: () => CONST_1240 / yMinLambda.value,
    set: val => {
        // console.log('max energy input: ' + val)
        yMinLambda.value = CONST_1240 / val;
    }
});
const yMinIndex = computed(() => {
    let min = dataSnapshot.value.wavelength[0];
    let max = dataSnapshot.value.wavelength[dataSnapshot.value.width - 1];
    return Math.round((yMinLambda.value - min) / (max - min) * (dataSnapshot.value.width - 1));
});
const yMaxIndex = computed(() => {
    let min = dataSnapshot.value.wavelength[0];
    let max = dataSnapshot.value.wavelength[dataSnapshot.value.width - 1];
    return Math.round((yMaxLambda.value - min) / (max - min) * (dataSnapshot.value.width - 1));
});

watch(() => prop.data, newData => { 
    if(chart1) {
        chart1.dispose();
    }
    nextTick().then(() => {
        chart1 = echarts.init(arspc.value! as HTMLDivElement);
        if(newData) {
            drawARSpec(chart1, dataSnapshot.value, prop.name, 0, prop.data!.height - 1);
        }
        yAxisMode.value = true;
        xMinIndex.value = 0;
        xMaxIndex.value = newData!.height - 1;
        yMinLambda.value = newData!.wavelength[0];
        yMaxLambda.value = newData!.wavelength[newData!.width - 1];
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

watch(xAxisIsTan, newBool => {
    let xAxisOptions = (chart1.getOption().xAxis as {[key: string]: any}[]);
    if(newBool) {
        xAxisOptions[1].name = "tan(θ)";
        xAxisOptions[1].min = Math.tan(xMinAngle.value / 180 * Math.PI);
        xAxisOptions[1].max = Math.tan(xMaxAngle.value / 180 * Math.PI);
        xAxisOptions[1].axisLabel.formatter = (value: number) => value.toFixed(2);
    } else {
        xAxisOptions[1].name = "Index";
        xAxisOptions[1].min = xMinIndex.value;
        xAxisOptions[1].max = xMaxIndex.value;
        xAxisOptions[1].axisLabel.formatter = (value: number) => value.toFixed(0);
    }
    chart1.setOption({
        xAxis: xAxisOptions
    })
})

watch(xMinIndex, newIndex => {
    let xAxisOptions = (chart1.getOption().xAxis as {[key: string]: any}[]);
    let min = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value.height - 1));
    xAxisOptions[0].min = min;
    if(!xAxisIsTan.value) {
        xAxisOptions[1].min = newIndex;
    }
    chart1.setOption({ xAxis: xAxisOptions });
});

watch(xMaxIndex, newIndex => {
    let xAxisOptions = (chart1.getOption().xAxis as {[key: string]: any}[]);
    let max = Math.round(newIndex / (prop.data!.height - 1) * (dataSnapshot.value.height - 1));
    xAxisOptions[0].max = max;
    if(!xAxisIsTan.value) {
        xAxisOptions[1].max = newIndex;
    }
    chart1.setOption({ xAxis: xAxisOptions });
});

watch(xMinAngle, newAngle => {
    if(xAxisIsTan.value) {
        let xAxisOptions = (chart1.getOption().xAxis as {[key: string]: any}[]);
        xAxisOptions[1].min = Math.tan(newAngle / 180 * Math.PI);;
        chart1.setOption({ xAxis: xAxisOptions });
    }
});

watch(xMaxAngle, newAngle => {
    if(xAxisIsTan.value) {
        let xAxisOptions = (chart1.getOption().xAxis as {[key: string]: any}[]);
        xAxisOptions[1].max = Math.tan(newAngle / 180 * Math.PI);
        chart1.setOption({ xAxis: xAxisOptions });
    }
});

watch(yMinIndex, newIndex => {
    let yAxisOptions = (chart1.getOption().yAxis as {[key: string]: any}[]);
    if(yAxisMode.value) {
        yAxisOptions[1].min = yMinLambda.value;
    } else {
        yAxisOptions[1].max = yMaxEnergy.value;
    }
    yAxisOptions[0].min = newIndex;
    chart1.setOption({
        yAxis: yAxisOptions
    })
})

watch(yMaxIndex, newIndex => {
    let yAxisOptions = (chart1.getOption().yAxis as {[key: string]: any}[]);
    if(yAxisMode.value) {
        yAxisOptions[1].max = yMaxLambda.value;
    } else {
        yAxisOptions[1].min = yMinEnergy.value;
    }
    yAxisOptions[0].max = newIndex;
    chart1.setOption({
        yAxis: yAxisOptions
    })
})

watch(yAxisMode, newVal => {
    let yAxisOptions = (chart1.getOption().yAxis as {[key: string]: any}[]);
    // console.log(`${yMinIndex.value}, ${yMaxIndex.value}`)
    if(newVal) {
        // let yData = dataSnapshot.value.wavelength;
        // yAxisOptions[0].data = yData;
        yAxisOptions[0].inverse = false;
        yAxisOptions[1].min = yMinLambda.value;
        yAxisOptions[1].max = yMaxLambda.value;
        yAxisOptions[1].name = "Wavelength (nm)";
        yAxisOptions[1].axisLabel.formatter = (value: number) => value.toFixed(0);
    } else {
        // let yData = dataSnapshot.value.wavelength.map(lambda => CONST_1240 / lambda);
        // yAxisOptions[0].data = yData;
        yAxisOptions[0].inverse = true;
        yAxisOptions[1].min = yMinEnergy.value;
        yAxisOptions[1].max = yMaxEnergy.value;
        yAxisOptions[1].name = "Energy (eV)";
        yAxisOptions[1].axisLabel.formatter = (value: number) => value.toFixed(2);
    }
    chart1.setOption({
        yAxis: yAxisOptions
    })
})

const showOptions = ref(false);

function resetYRange() {
    yMinLambda.value = dataSnapshot.value.wavelength[0];
    yMaxLambda.value = dataSnapshot.value.wavelength[dataSnapshot.value.width - 1];
}
function resetXRange() {
    xMinIndex.value = 0;
    xMaxIndex.value = prop.data!.height - 1;
}
function resetXBinding() {
    xMinAngle.value = '';
    xMaxAngle.value = '';
}

const emit = defineEmits(['copy-to-clipboard']);

function copyToClipboard() {
    emit('copy-to-clipboard', '数据已复制到剪贴板，可在Origin直接粘贴表格');

    let str = `${xAxisIsTan.value? 'tan(θ)': 'Index'}\t${yAxisMode.value? 'Wavelength': 'Energy'}\tcounts
                \t${yAxisMode.value? 'nm': 'eV'}\n\n`;
    if(xAxisIsTan.value) {
        let minTan = Math.tan(xMinAngle.value / 180 * Math.PI);
        let maxTan = Math.tan(xMaxAngle.value / 180 * Math.PI);
        let height = xMaxIndex.value - xMinIndex.value + 1;
        let d = (maxTan - minTan) / (height - 1);
        for(var a = 0; a < height; a ++) {
            str += `\t${minTan + a * d}`;
        }
    } else {
        for(var b = xMinIndex.value; b <= xMaxIndex.value; b ++) {
            str += `\t${b}`;
        }
    }
    str += '\n';
    let min = prop.data!.wavelength[0];
    let max = prop.data!.wavelength[prop.data!.width - 1];
    let yMin = Math.round((yMinLambda.value - min) / (max - min) * (prop.data!.width - 1));
    let yMax = Math.round((yMaxLambda.value - min) / (max - min) * (prop.data!.width - 1));
    for(var i = yMin; i <= yMax; i ++) {
        str += `${yAxisMode.value? prop.data!.wavelength[i]: (CONST_1240 / prop.data!.wavelength[i])}`;
        for(var j = xMinIndex.value; j <= xMaxIndex.value; j ++) {
            str += `\t${prop.data!.frame[0][j * prop.data!.width + i]}`;
        }
        str +='\n';
    }
    writeText(str);
}
</script>

<template>
    <div id="arspc-container">
        <div ref="arspc" id="ar-spectrum"></div>
        <div id="arspc-tools">
            <button @click="showOptions = !showOptions" title="选项">
                <IconOption v-if="!showOptions" />
                <IconArrow v-if="showOptions" />
            </button>
            <button style="grid-row-start: 3;" @click="copyToClipboard" title="复制数据到剪贴板">
                <IconCopy/>
            </button>
            <button style="grid-row-start: 4;" @click="saveImage(chart1, `角分辨光谱-${prop.name}`)" title="导出图片">
                <IconExport/>
            </button>
        </div>
        <Transition>
            <div id="arspc-options" v-show="showOptions">
                <ModeSwitch :name="'纵轴模式'" :mode1="'波长'" :mode2="'能量'" v-model="yAxisMode" />
                <div class="range-input-lambda" v-show="yAxisMode">
                    <label for="y-min-lambda">纵轴范围</label>
                    <button @click="resetYRange"><IconReset/></button>
                    <input id="y-min-lambda" type="text" v-model.number="yMinLambda">
                    <label for="y-max-lambda">~</label>
                    <input id="y-max-lambda" type="text" v-model.number="yMaxLambda">
                </div>
                <div class="range-input-energy" v-show="!yAxisMode">
                    <label for="y-min-energy">纵轴范围</label>
                    <button @click="resetYRange"><IconReset/></button>
                    <input id="y-min-energy" type="text" v-model.number="yMinEnergy">
                    <label for="y-max-energy">~</label>
                    <input id="y-max-energy" type="text" v-model.number="yMaxEnergy">
                </div>
                <div class="range-input-x">
                    <label for="x-min-index">横轴范围</label>
                    <button @click="resetXRange"><IconReset/></button>
                    <input id="x-min-index" type="number" v-model.number="xMinIndex">
                    <label for="x-max-index">~</label>
                    <input id="x-max-index" type="number" v-model.number="xMaxIndex">
                    <p>(索引)</p>
                    <label for="x-min-angle" style="grid-column-start: 1">横轴绑定</label>
                    <button @click="resetXBinding"><IconReset/></button>
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
    background-color: rgb(223, 223, 223);
    border-radius: 3mm;
    display: flex;
    transition: 0.3s;
}

#arspc-container:hover {
    filter: drop-shadow(0px 0px 4px rgba(0, 0, 0, 0.1))
}

#ar-spectrum {
    background-color: white;
    width: 12cm;
    height: 16cm;
    border-radius: 1mm;
}

#arspc-tools {
    display: grid;
    grid-template-columns: auto;
    grid-template-rows: auto 1fr auto auto;
    gap: 1mm
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
    flex-direction: column;
    padding: 1mm;
    justify-content: left;
    gap: 1mm;
    padding-top: 6mm;
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