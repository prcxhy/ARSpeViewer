---
name: arspe-viewer
description: 使用 arspe_py（ARSpeViewer 的 Python 绑定）解析和处理角分辨光谱数据。当用户提到 .spe/.asc/.txt 光谱文件、角分辨光谱、ARSpeViewer、波长↔能量换算、tanθ/角度/波矢 k 轴、光谱数据批量处理或分析脚本时使用——即使用户没有明确提到"Python 接口"或这个包的名字。
---

# arspe_py：角分辨光谱数据处理

ARSpeViewer 的 Python 绑定。解析 `.spe`（二进制）/`.asc`/`.txt`/`.csv`（文本）光谱
文件，做波长↔能量、tanθ↔角度↔波矢换算与坐标轴插值拉伸。与桌面版 GUI 共用同一份
Rust 核心代码，数值口径一致。

## 环境检查（先做这一步）

```bash
.venv/Scripts/python.exe -c "import arspe_py; print(arspe_py.__version__)"   # Windows Git Bash
```

导入失败时构建（需 Rust 工具链 + numpy + maturin）：

```bash
source .venv/Scripts/activate && cd crates/arspe-py && maturin develop --release
```

maturin 因网络失败时的兜底：`cargo build --release -p arspe-py`，然后把
`target/release/arspe_py.dll` 改名为 `arspe_py.pyd` 拷入 `.venv/Lib/site-packages/`。

## API 速查

```python
import arspe_py as arspe
import numpy as np

spe = arspe.open_file(path)            # 或 parse_spe(bytes) / parse_text(str)
spe.frames                             # np.ndarray (帧数, height, width) float64
spe.wavelength                         # (width,) 波长轴 nm，无标定时为 None（先判空！）
spe.calibration                        # 六个标定参数
st = arspe.stretch(spe, ev_mode=False, x_mode="tan", tan_min=-0.4, tan_max=0.4)
arspe.lambda_to_energy(nm) / energy_to_lambda(ev)     # λ·E ≈ 1239.84
arspe.tan_to_angle / angle_to_tan / tan_to_k / k_to_tan
arspe.YRange(min_nm, max_nm) / arspe.XRange(na, lambda_nm, binding=None)
spe.to_dict() / spe.to_json() / spe.save_csv(path, frame_index=0)
```

异常：`arspe.ArspeError` 基类，`ParseError` / `UnsupportedFormatError` / `StretchError`，
错误消息为中文，可直接透出。

## 必须知道的数据语义

- `frames[frame, x, y]`：**x = 角度行（height 方向），y = 波长列（width 方向）**。
  用三维索引，不要自己换算展平下标 `x * width + y`。
- `stretch` 的 x_mode：`"tan"` 模式 x 向数值恒等（只用于 eV 纵轴重映射）；`"angle"`
  按角度均匀重建；`"k"` 按波矢重建。**`ev_mode=True` 和 `x_mode="k"` 都需要波长数据**，
  缺失抛 `StretchError`。
- 横轴裁剪：Python 侧用 numpy 切片 `frames[:, x0:x1, y0:y1]`，没有内置裁剪函数。
- **没有 `downsample`**：降采样是 GUI 渲染专用，Python 侧不存在这个 API。

## 常见坑

- 处理来源不明的 `.spe` 前先检查文件大小 > 4100 字节（严重截断的文件会触发 Rust
  panic → `pyo3_runtime.PanicException`，见 ISSUES #7）。
- `frames` 属性每次访问从 Rust 拷贝一份新数组；循环中先存局部变量。
- 波长子区间找帧内索引用 `YRange.index_range_of_lambda_in(full, spe.width)`（能量轴
  方向与波长相反，用 `index_range_of_energy_in`）。

## 深入阅读

完整 API 语义、绑定/区间映射公式、matplotlib 工作流示例：
读 `docs/python-api.md`。修改 Rust core 后的回归：`cargo test --workspace` +
`python -m pytest crates/arspe-py/tests/ -q` + `python .session/py-parity.py`。
真实样例在 `.session/fixtures/`（100.spe、power_3.asc），可作冒烟测试输入。
