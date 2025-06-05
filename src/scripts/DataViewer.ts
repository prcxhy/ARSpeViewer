import { path } from '@tauri-apps/api';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { ECharts } from 'echarts';

const H = 6.62607015e-34;
const C0 = 299792458;
const e = 1.602176634e-19;
const CONST_1240 = H * C0 * 1e9 / e;

class SpeData {
    min_max: number[][];
    frame: number[][];
    width: number;
    height: number;
    wavelength: number[];
    detector_angle_cal: number;
    focal_length_cal: number;
    inclusion_angle_cal: number;
    detector_angle_exp: number;
    focal_length_exp: number;
    inclusion_angle_exp: number;
    constructor(
        min_max: number[][],
        frame: number[][],
        width: number,
        height: number,
        wavelength: number[],
        detector_angle_cal: number,
        focal_length_cal: number,
        inclusion_angle_cal: number,
        detector_angle_exp: number,
        focal_length_exp: number,
        inclusion_angle_exp: number
    ) {
        this.min_max = min_max;
        this.frame = frame;
        this.width = width;
        this.height = height;
        this.wavelength = wavelength;
        this.detector_angle_cal = detector_angle_cal;
        this.focal_length_cal = focal_length_cal;
        this.inclusion_angle_cal = inclusion_angle_cal;
        this.detector_angle_exp = detector_angle_exp;
        this.focal_length_exp = focal_length_exp;
        this.inclusion_angle_exp = inclusion_angle_exp;

    }

}

function downSampling(speData: SpeData, stride: number): SpeData {
    if (stride <= 1) {
        return speData;
    }
    let snapshot = { ...speData };
    snapshot.width = Math.ceil(speData.width / stride);
    snapshot.height = Math.ceil(speData.height / stride);
    snapshot.wavelength = [];
    for (var i = 0; i < speData.width; i += stride) {
        snapshot.wavelength.push(speData.wavelength[i]);
    }
    snapshot.frame = speData.frame.map(oneFrame => {
        let newFrame: number[] = [];
        for (var i = 0; i < speData.height; i += stride) {
            for (var j = 0; j < speData.width; j += stride) {
                newFrame.push(oneFrame[i * speData.width + j]);
            }
        }
        return newFrame;
    })
    return snapshot;
}

async function drawARSpec(chart: ECharts, speData: SpeData, name: string, xMin: number, xMax: number) {
    let data: number[][] = [];
    let xLength = speData.height;
    let yLength = speData.width;
    let xData = Array.from({ length: xLength }, (_, i) => i);
    let yData = speData.wavelength;
    let zMin = speData.min_max[0][0];
    let zMax = speData.min_max[0][1];

    for (var j = 0; j < yLength; j++) {
        for (var i = 0; i < xLength; i++) {
            data.push([i, j, speData.frame[0][i * yLength + j]])
        }
    }
    chart.setOption({
        title: {
            text: name, left: 'center', top: 7,
            textStyle: { color: "#000", fontSize: 16, fontFamily: 'Arial' }
        },
        tooltip: {
            textStyle: { fontSize: 14 }, padding: [0, 4],
            formatter: (params: { [key: string]: any }) => {
                let num = params.value[2];
                return `${params.seriesName}<br/>${num}`;
            },
            // axisPointer: {
            //     axis: 'x'
            // },
            // trigger: 'axis', triggerOn: 'click', showContent: false
        },
        grid: {
            // show: true,
            // backgroundColor: 'rgb(255, 255, 255)',
            top: 35, bottom: 49, right: 98, left: 70
        },
        xAxis: [
            {
                show: false,
                type: 'category',
                position: 'bottom',
                data: xData,
            },
            {
                show: true,
                type: 'value',
                position: 'bottom',
                min: xMin,
                max: xMax,
                nameLocation: 'center', nameGap: 28,
                name: "Index", nameTextStyle: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 16
                },
                axisTick: {
                    show: true, color: "#000"
                },
                axisLine: {
                    show: false
                },
                axisLabel: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 14,
                    formatter: (value: number) => {
                        return value.toFixed(0);
                    }
                },
            },
            {
                show: false,
                type: 'value',
                position: 'bottom',
                min: xMin,
                max: xMax,
                nameLocation: 'center', nameGap: 28,
                name: "tan(θ)", nameTextStyle: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 16
                },
                axisTick: {
                    show: true, color: "#000"
                },
                axisLine: {
                    show: false
                },
                axisLabel: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 14,
                    formatter: (value: number) => {
                        return value.toFixed(2);
                    }
                },
            },
        ],
        yAxis: [
            {
                show: false,
                type: 'category',
                position: 'right',
                data: yData,
            },
            {
                type: 'value',
                position: 'left',
                min: yData[0],
                max: yData[yLength - 1],
                nameLocation: 'center', nameGap: 42,
                name: "Wavelength (nm)", nameTextStyle: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 16
                },
                axisTick: {
                    show: true, color: "#000"
                },
                axisLabel: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 14,
                    formatter: (value: number) => {
                        return value.toFixed(0);
                    }
                },
            },
            {
                show: false,
                type: 'value',
                position: 'left',
                min: CONST_1240 / yData[yLength - 1],
                max: CONST_1240 / yData[0],
                nameLocation: 'center', nameGap: 42,
                name: "Energy (eV)", nameTextStyle: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 16
                },
                axisTick: {
                    show: true, color: "#000"
                },
                axisLabel: {
                    color: "#000", fontFamily: 'Times New Roman', fontSize: 14,
                    formatter: (value: number) => {
                        return value.toFixed(2);
                    }
                },
            },
        ],
        dataZoom: [{
            type: 'slider',
            xAxisIndex: 0,
            show: false
        }, {
            type: 'slider',
            yAxisIndex: 0,
            show: false
        }, {
            type: 'slider',
            xAxisIndex: 1,
            show: false
        }, {
            type: 'slider',
            yAxisIndex: 1,
            show: false
        }, {
            type: 'slider',
            yAxisIndex: 2,
            show: false
        }],
        visualMap: {
            min: zMin,
            max: zMax,
            calculable: true,
            realtime: false,
            textStyle: {
                color: "#000", fontFamily: 'Times New Roman', fontSize: 14
            },
            handleStyle: {
                opacity: 0
            },
            inRange: {
                color: [
                    // plasma
                    '#0d0887',
                    '#5301a8',
                    '#9E0698',
                    '#DF3753',
                    '#FD9927',
                    '#F0F921'

                    // viridis
                    // '#440154',
                    // '#414487',
                    // '#2A788E',
                    // '#22A884',
                    // '#7AD151',
                    // '#FDE725'
                ]
            },
            itemHeight: 256,
            right: 7, top: 'center',
            align: 'left',
            formatter: (value: string) => {
                return parseFloat(value).toExponential(1)
            }
        },
        series: [
            {
                name: "counts",
                type: 'heatmap',
                data: data,
                emphasis: {
                    itemStyle: {
                        borderColor: '#0f0',
                        borderWidth: 1
                    }
                },
                progressive: 1000,
                animation: false
            }
        ]
    })
}
async function drawSpec(chart: ECharts, speData: SpeData, index: number, name: string) {
    let data = speData.wavelength.map((lambda, j) => {
        return [lambda, speData.frame[0][index * speData.width + j]]
    });
    chart.setOption({
        animation: false,
        tooltip: {
            trigger: 'item', padding: [0, 4],
            formatter: (params: { [key: string]: any }) => {
                return `${(params.value[0] as number).toFixed(2)} nm<br/>counts: ${params.value[1]}`;
            },
            textStyle: { fontSize: 14 }
        },
        legend: {
            top: 7, right: 'center',
            textStyle: {
                color: "#000", fontFamily: 'Arial', fontSize: 16
            },
        },
        dataZoom: [
            { type: 'inside', yAxisIndex: 0 },
            { type: 'inside', xAxisIndex: 0 }
        ],
        grid: {
            show: true,
            borderColor: "#000",
            backgroundColor: '#fff',
            top: 35, bottom: 49, right: 14, left: 70
        },
        xAxis: {
            type: 'value', nameLocation: 'center', nameGap: 28,
            name: "Wavelength (nm)", nameTextStyle: {
                color: "#000", fontFamily: 'Times New Roman', fontSize: 16
            },
            min: speData.wavelength[0],
            max: speData.wavelength[speData.width - 1],
            axisLabel: {
                color: "#000", fontFamily: 'Times New Roman', fontSize: 14,
            },
            splitLine: { show: false },
        },
        yAxis: {
            type: 'value', nameLocation: 'center', nameGap: 42,
            name: "counts", nameTextStyle: {
                color: "#000", fontFamily: 'Times New Roman', fontSize: 16
            },
            axisLabel: {
                color: "#000", fontFamily: 'Times New Roman', fontSize: 14,
                formatter: (value: number) => {
                    return value.toExponential();
                }
            },
            splitLine: { show: false },
        },
        series: {
            name: name, type: 'line', data: data,
            symbol: 'none', lineStyle: { width: 3 }
        }
    })

}

async function saveImage(chart: echarts.ECharts, name: string, isntDrawn: boolean) {
    if(isntDrawn) { return }
    let picturePath = await path.pictureDir();
    let defaultPath = await path.join(picturePath, name);
    let savePath = await save({
        defaultPath: defaultPath,
        filters: [{
            name: '*.png 图片',
            extensions: ['png'],
        }]
    })
    if(savePath) {
        let url = chart.getDataURL({
            type: 'png',
            pixelRatio: 3,
            backgroundColor: '#fff'
        });
        let binaryString = atob(url.split(',')[1]);
        let u8Array = new Uint8Array(binaryString.length);
        for (var i = 0; i < binaryString.length; i++) {
            u8Array[i] = binaryString.charCodeAt(i);
        }
        writeFile(savePath, u8Array);
        return '已将图片保存到 ' + savePath
    }
}

export { CONST_1240, SpeData, downSampling, drawARSpec, drawSpec, saveImage }