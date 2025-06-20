import { CONST_1240 } from "./DataViewer";

class YRange {
    private _minLambda: number;
    private _maxLambda: number;
    constructor(wavelength: number[]) {
        this._minLambda = wavelength[0];
        this._maxLambda = wavelength[wavelength.length - 1];
    }
    get minLambda() {
        return this._minLambda
    }
    get maxLambda() {
        return this._maxLambda
    }
    get minEnergy() {
        return CONST_1240 / this._maxLambda
    }
    get maxEnergy() {
        return CONST_1240 / this._minLambda
    }
    set minLambda(val) {
        this._minLambda = val
    }
    set maxLambda(val) {
        this._maxLambda = val
    }
    set minEnergy(val) {
        this._maxLambda = CONST_1240 / val
    }
    set maxEnergy(val) {
        this._minLambda = CONST_1240 / val
    }
    get bandWidth() {
        return this._maxLambda - this._minLambda
    }
    get bandGap() {
        return CONST_1240 / this._minLambda - CONST_1240 / this._maxLambda
    }
    indexRangeOfEnergyIn(range: YRange, length: number) {
        return [
            Math.round((range.maxEnergy - this.maxEnergy) / range.bandGap * length),
            Math.round((range.maxEnergy - this.minEnergy) / range.bandGap * length)
        ]
    }
    indexRangeOfLambdaIn(range: YRange, length: number) {
        return [
            Math.round((this._minLambda - range.minLambda) / range.bandWidth * length),
            Math.round((this._maxLambda - range.minLambda) / range.bandWidth * length)
        ]
    }
}

class XRange {
    private _minTan: number;
    private _maxTan: number;
    private lambda: number;
    constructor(NA: number, lambda: number, binding?: number[]) {
        let tan = Math.tan(Math.asin(NA));
        if (binding) {
            let from = binding[0];
            let to = binding[1];
            let length = binding[2];
            this._minTan = -tan - 2 * tan / (to - from) * from;
            this._maxTan = this._minTan + 2 * tan / (to - from) * length;
        } else {
            this._maxTan = tan;
            this._minTan = -tan;
        }
        this.lambda = lambda;
    }
    get minTan() {
        return this._minTan
    }
    get maxTan() {
        return this._maxTan
    }
    set minTan(val) {
        this._minTan = val
    }
    set maxTan(val) {
        this._maxTan = val
    }
    get minAngle() {
        return Math.atan(this._minTan) / Math.PI * 180
    }
    get maxAngle() {
        return Math.atan(this._maxTan) / Math.PI * 180
    }
    set minAngle(val) {
        this._minTan = Math.tan(val * Math.PI / 180)
    }
    set maxAngle(val) {
        this._maxTan = Math.tan(val * Math.PI / 180)
    }
    get minK() {
        return 2 * Math.PI * this._minTan / this.lambda * 1000
    }
    get maxK() {
        return 2 * Math.PI * this._maxTan / this.lambda * 1000
    }
    set minK(val) {
        this._minTan =  val * this.lambda / (1000 * 2 * Math.PI)
    }
    set maxK(val) {
        this._maxTan =  val * this.lambda / (1000 * 2 * Math.PI)
    }
    indexRangeOfTanIn(range: XRange, length: number) {
        return [
            Math.round((this._minTan - range.minTan) / (range.maxTan - range.minTan) * length),
            Math.round((this._maxTan - range.minTan) / (range.maxTan - range.minTan) * length)
        ]
    }
    indexRangeOfAngleIn(range: XRange, length: number) {
        return [
            Math.round((this.minAngle - range.minAngle) / (range.maxAngle - range.minAngle) * length),
            Math.round((this.maxAngle - range.minAngle) / (range.maxAngle - range.minAngle) * length)
        ]
    }
    indexRangeOfKIn(range: XRange, length: number) {
        return [
            Math.round((this.minK - range.minK) / (range.maxK - range.minK) * length),
            Math.round((this.maxK - range.minK) / (range.maxK - range.minK) * length)
        ]
    }
}

export { YRange, XRange }