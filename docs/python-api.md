# arspe-py Python 接口指南

`arspe-py`（导入名 `arspe_py`）是 ARSpeViewer 核心数据处理能力的 Python 绑定，
基于 [PyO3](https://pyo3.rs) + [maturin](https://www.maturin.rs) 构建。它与桌面版
GUI **共用同一份 Rust 核心代码**（`crates/arspe-core`），解析结果与拉伸口径和
GUI 完全一致，并由三方对拍测试锁定（`.session/py-parity.py`）。

面向场景：无 GUI 的自动化工具流、批处理脚本、LLM Agent 编排。

## 环境构建

依赖：Python ≥ 3.10（abi3，无需按版本分别编译）、numpy ≥ 1.26、Rust 工具链。

```shell
# 推荐：venv + maturin develop（改 Rust 代码后重跑即可热更新绑定）
python -m venv .venv
source .venv/Scripts/activate        # Windows Git Bash；Linux/macOS 为 bin/activate
pip install numpy maturin pytest
cd crates/arspe-py && maturin develop --release
```

> 网络（crates.io）不可达时 maturin 会失败。兜底方案：`cargo build --release -p arspe-py`
> 直接构建，再把 `target/release/arspe_py.dll`（Linux 为 `.so`，macOS 为 `.dylib`）
> 改名为 `arspe_py.pyd` 拷入 venv 的 `site-packages/`。

验证安装：

```python
import arspe_py as arspe
arspe.__version__                    # '1.2.0'
arspe.lambda_to_energy(532.0)        # ≈ 2.33053 eV
```

## 数据模型：SpeData

```python
spe = arspe.open_file(path)          # 按扩展名分流（.spe → 二进制解析，其余 → 文本解析）
spe = arspe.parse_spe(data: bytes)   # 字节流输入（如 Agent 拿到 blob）
spe = arspe.parse_text(text: str)    # 文本输入（.txt/.csv/.asc，行/列主序自动识别）
```

| 属性 | 类型 | 语义 |
| --- | --- | --- |
| `frames` | `np.ndarray (n, height, width)` float64 | 全部帧；**height = 角度行（x 方向），width = 波长列（y 方向）**，即 `frames[frame, x_row, y_col]` |
| `wavelength` | `np.ndarray (width,)` 或 `None` | 波长轴（nm）。**文本数据与 .spe 无标定数据时为 None** |
| `min_max` | `np.ndarray (n, 2)` | 每帧 [min, max] |
| `width` / `height` / `frame_count` | int | 列数 / 行数 / 帧数 |
| `calibration` | `Calibration` | 六个标定参数（cal/exp 各三组：detector_angle、focal_length、inclusion_angle） |

数据布局与桌面版前后端 JSON 契约一致：原始 Rust 存储为逐帧展平的行主序
`frame[frame][x * width + y]`，绑定层已重组为三维 numpy 数组，**建议一律用
`frames[frame, x, y]` 索引，不要自己换算展平下标**。

导出：

```python
spe.to_dict()                        # dict，字段与 GUI JSON 契约一致（嵌套 list）
spe.to_json()                        # str，同上
spe.save_csv(path, frame_index=0, header=True)
# 单帧 → 制表符分隔矩阵：行 = 波长（无波长则行索引），列 = 角度行索引
# 可直接粘贴到 Origin / Excel；注意仅导出单帧，多帧需循环调用
```

## 坐标轴拉伸：stretch

```python
st = arspe.stretch(spe, ev_mode: bool, x_mode: str, tan_min: float, tan_max: float)
```

- `ev_mode=True`：纵轴（波长）按能量 eV 等间隔重建，返回数据的 `wavelength` 变为
  非线性重映射的波长轴（首尾不变）。**需要波长数据**，否则抛 `StretchError`。
- `x_mode`：
  - `"tan"`：横轴保持 tanθ 均匀网格。求值点与节点重合，**x 向数值恒等**（只服务于
    eV 纵轴重映射或作为其他模式的基线）；
  - `"angle"`：横轴按角度均匀重建（tanθ 网格边缘被压缩），需要波长仅用于 k 换算时；
  - `"k"`：横轴按波矢 k (μm⁻¹) 重建，k = 2π·tanθ / λ。**需要波长数据**；
  - `tan_min` / `tan_max` 是 tanθ 范围，angle/k 模式内部自动换算。
- 返回新 `SpeData`，输入对象不被修改；插值为 2D 线性 + `Extrapolate::Clamp`（边界钳制），
  与 GUI `stretch` 命令同源。

注意：横轴裁剪（GUI 中的"同步裁剪"）在桌面版里由前端 dataZoom 完成，Python 侧请用
numpy 切片 `frames[:, x0:x1, y0:y1]` 自行裁剪。

## 基本换算函数

| 函数 | 说明 |
| --- | --- |
| `lambda_to_energy(nm)` / `energy_to_lambda(ev)` | λ·E ≈ 1239.84（CODATA 物理常数计算，仅标量）；**数组换算用常数 `arspe.CONST_1240 / wl_array`** |
| `tan_to_angle(t)` / `angle_to_tan(deg)` | tanθ ↔ 角度（°） |
| `tan_to_k(t, lambda_nm)` / `k_to_tan(k, lambda_nm)` | k (μm⁻¹) = 2π·tanθ / λ；k 换算的 λ 语义上取波长轴末端（最大波长），与 GUI 一致 |
| `CONST_1240` | 暴露的物理常数（float），供 numpy 数组向量化换算 |

## 区间与索引映射：YRange / XRange

对应前端 `ParametersConvert.ts` 的 TS 实现（golden 对拍一致），用于"给定子区间在
全局轴上的索引位置"类问题（GUI 缩放/裁剪的核心数学）：

```python
full = arspe.YRange(400.0, 1000.0)           # 波长区间（nm）
full.min_energy, full.max_energy             # 能量视图（eV），可读写（setter 联动）
full.band_width, full.band_gap
lo, hi = sub.index_range_of_lambda_in(full, 100)    # 子区间映射到全局的索引范围
lo, hi = sub.index_range_of_energy_in(full, 100)    # 能量视角（注意能量轴方向与波长相反）

xr = arspe.XRange(na=0.5, lambda_nm=850.0)          # 全域 ±tan(asin(NA))
xr = arspe.XRange(0.5, 850.0, binding=(frm, to, length))  # TS 绑定模式：索引窗口 [frm,to]
xr.min_tan / min_angle / min_k ...                  # 三视图互相换算（可读写）
lo, hi = xr.index_range_of_tan_in(full_range, 100)  # 另有 angle / k 版本
```

`binding=(from, to, length)` 语义沿用前端公式：
`minTan = -tan - 2·tan/(to-from)·from`，`maxTan = minTan + 2·tan/(to-from)·length`。

## 异常处理

```
ArspeError                      # 基类
├── ParseError                  # 文件损坏 / 内容无法解析（含：路径不存在的 .spe）
├── UnsupportedFormatError      # 无法读取的文件（非 UTF-8 文本、目录路径等）
└── StretchError                # 拉伸/换算失败（eV、k 模式缺波长；插值失败）
```

错误消息为中文（与桌面版 UI 文案一致），Agent 可直接透出给用户。

**已知风险**：`.spe` 解析按固定偏移取字节，**严重截断（< 4100 字节）的文件会触发
Rust panic**，Python 侧表现为 `pyo3_runtime.PanicException`（进程不崩，但报错不友好，
见 ISSUES #7）。Agent 处理来源不明的 .spe 文件时建议先检查
`len(data) > 4100` 或捕获 `Exception` 兜底。

## 典型工作流

### 批量扫描目录并汇总

```python
import numpy as np
from pathlib import Path
import arspe_py as arspe

rows = []
for p in sorted(Path("data/").glob("*.spe")):
    try:
        spe = arspe.open_file(p)
    except arspe.ArspeError as e:
        rows.append((p.name, "FAIL", str(e))); continue
    rows.append((p.name, "OK",
                 f"{spe.frame_count}帧 {spe.height}x{spe.width} 波长={spe.wavelength is not None}"))
```

### eV 拉伸 + matplotlib 绘图

```python
import matplotlib.pyplot as plt
import arspe_py as arspe

spe = arspe.open_file("run01.spe")
st = arspe.stretch(spe, ev_mode=True, x_mode="tan", tan_min=-0.4, tan_max=0.4)
ev = arspe.CONST_1240 / st.wavelength              # (width,) 能量轴（数组换算用常数除法）
tan = np.linspace(-0.4, 0.4, st.height)             # 横轴（tan 模式均匀网格）
plt.pcolormesh(tan, ev, st.frames[0].T)             # 注意转置：行列 ↔ xy
plt.xlabel(r"$\tan\theta$"); plt.ylabel("Energy (eV)")
```

### 波长子区间 → 帧内索引切片

```python
full = arspe.YRange.from_wavelength(spe.wavelength) if spe.wavelength is not None else None
sub = arspe.YRange(450.0, 550.0)
y0, y1 = sub.index_range_of_lambda_in(full, spe.width)   # 近似索引（四舍五入）
block = spe.frames[0][:, max(y0, 0):min(y1 + 1, spe.width)]
```

## Agent 注意事项

1. **数据口径**：本包与桌面版共享同一 Rust 实现，数值结果可直接对照 GUI；不要在
   Python 侧重写解析/插值逻辑。
2. **无 `downsample`**：降采样是 GUI 渲染专用（前端 TS 实现），Python 侧无此需求；
   需要缩分辨率请用 numpy 切片/采样。
3. **wavelength 为 None 的分支**：无标定的数据不支持 eV/k 轴，相关调用会抛
   `StretchError`；写批处理时先判空。
4. **性能**：重计算（解析、拉伸）在 Rust 侧并行执行并释放 GIL；`frames` 属性每次
   访问都会从 Rust 拷贝一份 numpy 数组（安全但非零开销），循环中请先存局部变量。
5. **回归验证**：修改 core 后运行 `cargo test --workspace`、
   `python -m pytest crates/arspe-py/tests/ -q` 与
   `python .session/py-parity.py`（TS↔Rust↔Python 对拍）。
6. **版本**：绑定版本与 app 对齐（`arspe.__version__`），当前 1.2.0；尚未发布
   预编译 wheel，仅供源码构建。
