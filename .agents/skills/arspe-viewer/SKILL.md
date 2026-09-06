---
name: arspe-viewer
description: 使用 arspe_py（ARSpeViewer 应用自带的 Python 绑定）解析和处理角分辨光谱数据。当用户提到 .spe/.asc/.txt/.csv 光谱文件、角分辨光谱、ARSpeViewer、波长↔能量换算、tanθ/角度/波矢 k 轴、光谱数据批量处理或分析脚本时使用——即使用户没有明确提到"Python 接口"或这个包的名字。
---

# arspe_py：角分辨光谱数据处理（ARSpeViewer 随包接口）

ARSpeViewer 桌面应用自带的 Python 数据接口（PyO3 扩展），与 GUI 共用同一份 Rust
核心代码，数值口径与桌面版完全一致。接口文件位于
`<ARSPEVIEWER_INSTALL_DIR>\py-interface\`（`arspe_py.pyd` 扩展 + `arspe_py.pyi`
类型存根 + README.txt）。

## 环境检查（先做这一步）

需要 Python 3.10+（abi3 通用）与 numpy（缺失则 `pip install numpy`）。把接口目录
插到 `sys.path[0]` 后导入——**必须 insert(0)**，若解释器环境里恰好也装过同名包，
普通 import 会优先命中 site-packages 那份：

```python
import sys; sys.path.insert(0, r"<ARSPEVIEWER_INSTALL_DIR>\py-interface")
import arspe_py
print(arspe_py.__version__, arspe_py.__file__)
# __file__ 应指向 <ARSPEVIEWER_INSTALL_DIR>\py-interface\arspe_py.pyd
```

## API 速查

```python
import arspe_py as arspe
import numpy as np

spe = arspe.open_file(path)            # 或 parse_spe(bytes) / parse_text(str)
spe.frames                             # np.ndarray (帧数, height, width) float64
spe.wavelength                         # (width,) 波长轴 nm，无标定时为 None（先判空！）
spe.calibration                        # 六个标定参数
spe.min_max                            # (帧数, 2) 每帧 [min, max]
st = arspe.stretch(spe, ev_mode=False, x_mode="tan", tan_min=-0.4, tan_max=0.4)
arspe.lambda_to_energy(nm) / energy_to_lambda(ev)      # λ·E ≈ 1239.84（仅标量）
arspe.CONST_1240                                       # 数组换算用 CONST_1240 / wl
arspe.tan_to_angle(t) / angle_to_tan(deg)              # tanθ ↔ 角度(°)
arspe.tan_to_k(t, lambda_nm) / k_to_tan(k, lambda_nm)  # k(μm⁻¹) = 2π·tanθ/λ
arspe.YRange(min_nm, max_nm) / arspe.XRange(na, lambda_nm, binding=None)
spe.to_dict() / spe.to_json() / spe.save_csv(path, frame_index=0)
```

异常：`arspe.ArspeError` 基类，`ParseError` / `UnsupportedFormatError` /
`StretchError`，错误消息为中文，可直接透出给用户。

## 必须知道的数据语义

- `frames[frame, x, y]`：**x = 角度行（height 方向），y = 波长列（width 方向）**。
  用三维索引，不要自己换算展平下标 `x * width + y`。
- `stretch` 的 x_mode：`"tan"` 模式 x 向数值恒等（只用于 eV 纵轴重映射）；`"angle"`
  按角度均匀重建（tanθ 网格边缘压缩）；`"k"` 按波矢重建。**`ev_mode=True` 和
  `x_mode="k"` 都需要波长数据**，缺失抛 `StretchError`。
- 横轴裁剪用 numpy 切片 `frames[:, x0:x1, y0:y1]`；**没有 `downsample` API**
  （降采样是 GUI 渲染专用，Python 侧不存在）。
- `save_csv` 导出单帧制表符矩阵（行 = 波长，列 = 角度行索引），可直接粘贴
  Origin / Excel；多帧需循环调用。
- 波长子区间找帧内索引用 `YRange.index_range_of_lambda_in(full, spe.width)`
  （能量轴方向与波长相反，用 `index_range_of_energy_in`）。

## 常见坑

- 处理来源不明的 `.spe` 前先检查文件大小 > 4100 字节（严重截断的文件会触发 Rust
  panic → `pyo3_runtime.PanicException`，进程不崩但报错不友好）。
- `frames` 属性每次访问从 Rust 拷贝一份新数组；循环中先存局部变量。
- 安装版接口随应用发布，先 `print(arspe_py.__version__)` 自检环境。

## 开发者附注（随包用户可忽略）

本文件是仓库 `.agents/skills/arspe-viewer/SKILL.md`，由 release CI 复制到安装目录
`skill/`；修改后需同步检查仓库 `docs/python-api.md`。源码开发与构建流程见仓库
`.session/PROJECT-NOTES.md`。
