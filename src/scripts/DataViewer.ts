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
    wavelength: number[] | null;
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
        wavelength: number[] | null,
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

function downSampling(speData: SpeData, size: number[]): SpeData {
    if (size[0] == speData.height && size[1] == speData.width) {
        return speData;
    }
    let snapshot = { ...speData };
    snapshot.width = Math.min(size[1], speData.width);
    snapshot.height = Math.min(size[0], speData.height);
    let dw = (speData.width - 1) / (size[1] - 1);
    let dh = (speData.height - 1) / (size[0] - 1);
    if (speData.wavelength) {
        snapshot.wavelength = [];
        for (var i = 0; i < snapshot.width; i ++) {
            snapshot.wavelength.push(speData.wavelength[Math.floor(i * dw)]);
        }
    }
    snapshot.frame = speData.frame.map(oneFrame => {
        let newFrame: number[] = [];
        for (var i = 0; i < snapshot.height; i ++) {
            for (var j = 0; j < snapshot.width; j ++) {
                newFrame.push(oneFrame[Math.floor(i * dh) * speData.width + Math.floor(j * dw)]);
            }
        }
        return newFrame;
    })
    return snapshot;
}

function initARSpec(
    chart: ECharts, speData: SpeData, xData: number[], yData: number[] | null, data: number[][],
    info: {
        name: string,
        yMin: number, yMax: number,
        xMin: number, xMax: number,
        zMin: number, zMax: number
    }) {

    let yLength = speData.width;

    chart.setOption({
        title: {
            text: info.name, left: 'center', top: 7,
            textStyle: { color: "#000", fontSize: 16, fontFamily: 'Arial' }
        },
        tooltip: {
            textStyle: { fontSize: 14 }, padding: [0, 4],
            formatter: (params: { [key: string]: any }) => {
                let num = params.value[2];
                return `${params.seriesName}<br/>${num}`;
            }
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
                min: info.xMin,
                max: info.xMax,
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
                min: info.xMin,
                max: info.xMax,
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
                min: yData ? yData[0] : info.yMin,
                max: yData ? yData[yLength - 1] : info.yMax,
                nameLocation: 'center', nameGap: 42,
                name: yData ? "Wavelength (nm)" : "Index",
                nameTextStyle: {
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
                min: yData ? CONST_1240 / yData[yLength - 1] : 0,
                max: yData ? CONST_1240 / yData[0] : yLength - 1,
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
            min: info.zMin,
            max: info.zMax,
            calculable: true,
            realtime: false,
            // hoverlink: false,
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
            itemHeight: chart.getHeight() - 84,
            right: 7, top: 23,
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
                // silent: true,
                emphasis: {
                    // disable: true,
                    itemStyle: {
                        borderColor: '#0f0',
                        borderWidth: 1
                    }
                },
                progressive: 2048,
                animation: false
            }
        ]
    })
}

async function drawARSpec(
    chart: ECharts, speData: SpeData,
    info: {
        name: string, frameIndex: number,
        yMin: number, yMax: number,
        xMin: number, xMax: number
    }
) {
    let data: number[][] = [];
    let xLength = speData.height;
    let yLength = speData.width;
    let xData = Array.from({ length: xLength }, (_, i) => i);
    let yData = speData.wavelength;
    let zMin = speData.min_max[info.frameIndex][0];
    let zMax = speData.min_max[info.frameIndex][1];

    for (var j = 0; j < yLength; j++) {
        for (var i = 0; i < xLength; i++) {
            data.push([i, j, speData.frame[info.frameIndex][i * yLength + j]])
        }
    }

    let option = chart.getOption();
    
    if (option) {
        (option.title as {[key: string]: any}[])[0].text = info.name;
        (option.xAxis as {[key: string]: any}[])[0].data = xData;
        (option.xAxis as {[key: string]: any}[])[1].min = info.xMin;
        (option.xAxis as {[key: string]: any}[])[1].max = info.xMax;
        (option.yAxis as {[key: string]: any}[])[0].data = yData;
        (option.yAxis as {[key: string]: any}[])[1].min = yData ? yData[0] : info.yMin;
        (option.yAxis as {[key: string]: any}[])[1].max = yData ? yData[yLength - 1] : info.yMax;
        (option.yAxis as {[key: string]: any}[])[1].name = yData ? "Wavelength (nm)" : "Index";
        (option.yAxis as {[key: string]: any}[])[2].min = yData ? CONST_1240 / yData[yLength - 1] : 0;
        (option.yAxis as {[key: string]: any}[])[2].max = yData ? CONST_1240 / yData[0] : yLength - 1;
        (option.visualMap as {[key: string]: any}[])[0].min = zMin;
        (option.visualMap as {[key: string]: any}[])[0].max = zMax;
        (option.visualMap as {[key: string]: any}[])[0].range = [zMin, zMax];
        (option.series as {[key: string]: any}[])[0].data = data;
        chart.setOption(option);
    } else {
        initARSpec(chart, speData, xData, yData, data, {...info, zMin: zMin, zMax: zMax});
    }
}

function initSpec(chart: ECharts, name: string, speData: SpeData, data: number[][]) {   
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
            min: speData.wavelength ? speData.wavelength[0] : 0,
            max: speData.wavelength ? speData.wavelength[speData.width - 1] : speData.width - 1,
            type: 'value', nameLocation: 'center', nameGap: 28,
            name: speData.wavelength ? "Wavelength (nm)" : "Index",
            nameTextStyle: {
                color: "#000", fontFamily: 'Times New Roman', fontSize: 16
            },
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

async function drawSpec(chart: ECharts, speData: SpeData, frameIndex: number, sliceIndex: number, name: string) {
    let data: number[][] = []
    for (var j = 0; j < speData.width; j++) {
        if (speData.wavelength) {
            data.push([speData.wavelength[j], speData.frame[frameIndex][sliceIndex * speData.width + j]]);
        } else {
            data.push([j, speData.frame[frameIndex][sliceIndex * speData.width + j]]);
        }
    }
    let option = chart.getOption();

    if (option) {
        (option.xAxis as {[key: string]: any}[])[0].min = speData.wavelength ? speData.wavelength[0] : 0;
        (option.xAxis as {[key: string]: any}[])[0].max = speData.wavelength ? speData.wavelength[speData.width - 1] : speData.width - 1;
        (option.xAxis as {[key: string]: any}[])[0].name = speData.wavelength ? "Wavelength (nm)" : "Index";
        (option.series as {[key: string]: any}[])[0].name = name;
        (option.series as {[key: string]: any}[])[0].data = data;
        chart.setOption(option);
    } else {
        initSpec(chart, name, speData, data);
    }
}

async function saveImage(chart: echarts.ECharts, name: string, isntDrawn: boolean, silentPath: string | undefined) {
    if (isntDrawn) { return }
    let savePath = silentPath ? await path.join(silentPath, `${name}.png`) : await save({
        defaultPath: name,
        filters: [{
            name: 'PNG',
            extensions: ['png'],
        }]
    })
    if (savePath) {
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

function compatible(newSpe: SpeData, oldSpe: SpeData, xMin: number, xMax: number, yMin: number, yMax: number) {
    let info: {
        yMin: number, yMax: number,
        xMin: number, xMax: number
    } = {
        yMin: 0, yMax: 0,
        xMin: 0, xMax: newSpe.height - 1
    };
    if (newSpe.wavelength && oldSpe.wavelength &&
        !(yMin >= newSpe.wavelength[newSpe.width - 1] || yMax <= newSpe.wavelength[0])) {
        let bandwidth = newSpe.wavelength[newSpe.width - 1] - newSpe.wavelength[0];
        if ((Math.abs(newSpe.wavelength[0] - oldSpe.wavelength[0]) <= bandwidth / 10) &&
        (Math.abs(newSpe.wavelength[newSpe.width - 1] - oldSpe.wavelength[oldSpe.width - 1]) <= bandwidth / 10)) {
            info.yMin = Math.max(yMin, newSpe.wavelength[0]);
            info.yMax = Math.min(yMax, newSpe.wavelength[newSpe.width - 1]);
        } else {
            info.yMin = newSpe.wavelength[0];
            info.yMax = newSpe.wavelength[newSpe.width - 1];
        }
    } else if ((!newSpe.wavelength && !oldSpe.wavelength) && newSpe.width == oldSpe.width) {
        info.yMin = yMin;
        info.yMax = yMax;
    } else if (!newSpe.wavelength) {
        info.yMin = 0;
        info.yMax = newSpe.width - 1;
    } else {
        info.yMin = newSpe.wavelength[0];
        info.yMax = newSpe.wavelength[newSpe.width - 1];
    }
    if (newSpe.height == oldSpe.height) {
        info.xMin = xMin;
        info.xMax = xMax;
    }
    return info;
}

export { CONST_1240, SpeData, downSampling, drawARSpec, drawSpec, saveImage, compatible }