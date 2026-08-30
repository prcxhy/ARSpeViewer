<h1 align="center">
  <img src="./src-tauri/icons/icon.png" alt="ARSpeViewer" width="128" />
  <br>
  <!-- <a href="https://github.com/prcxhy/ARSpeViewer">ARSpeViewer</a> -->
  ARSpeViewer
  <br>
</h1>
<h3 align="center">
(Angle-Resolved Spectral Viewer)<br>基于<a href="https://github.com/tauri-apps/tauri">Tauri</a>开发的角分辨光谱浏览器
</h3>

## 下载安装
支持 Windows 和 Mac OS([⚠️注意事项](#mac-os-使用注意))，请到发布页下载对应的安装包：[Release page](https://github.com/prcxhy/ARSpeViewer/releases)<br>

## 预览
![预览](./docs/preview.png)

## 功能
- 角分辨光谱数据文件(*.spe, *.asc, *.txt, *.csv)的读取
- 角分辨光谱heatmap交互式绘图
  - 多坐标轴模式切换，并基于插值准确mapping：**波长** *y* 轴、**能量** *y* 轴；**tan** *x* 轴、**角度** *x* 轴、**波矢*k*** *x* 轴
  - 即时响应的图像 & 数据**同步裁剪**
- 角度切片光谱即时响应绘图
  - 单击heatmap数据点可显示对应角度的光谱切片
- 光谱数据导出
  - 图片保存(⚠️仅供预览，勿代替正式科研绘图)
  - 复制数据到剪贴板：采用制表符分隔，**可直接粘贴**到 Origin 或 Excel

## Mac OS 使用注意
由于开发者没钱注册Apple开发者账号，无法给此应用合法签名，在Mac OS上安装后直接启动会提示文件损坏，采取以下步骤方可正常使用

1. 正常安装后，拖动安装文件夹(默认是**Applications**)至**终端**打开
2. 输入并回车执行以下命令:
   
   ```shell
   xattr -cr arspeviewer.app
   ```
3. 即可正常启动ARSpeViewer

## Python 接口（面向自动化脚本 / Agent）

核心数据提取与换算能力（文件解析、波长↔能量、tanθ↔角度↔波矢、坐标轴拉伸）已封装为
Python 扩展包 `arspe-py`（导入名 `arspe_py`），与桌面版共用同一份 Rust 核心代码，数据
口径完全一致，可无 GUI 运行：

```python
import arspe_py as arspe

spe = arspe.open_file("run01.spe")     # 也支持 parse_spe(bytes) / parse_text(str)
print(spe.frames.shape)                # (帧数, 角度行数, 波长列数) 的 numpy 数组

st = arspe.stretch(spe, ev_mode=True, x_mode="k", tan_min=-0.3, tan_max=0.3)
print(arspe.lambda_to_energy(532.0))   # 2.3305... eV
st.save_csv("run01_ev_k.csv")          # 制表符分隔，可直接粘贴到 Origin / Excel
```

当前需从源码构建（Python ≥ 3.10，需 Rust 工具链）：

```shell
pip install numpy maturin
cd crates/arspe-py && maturin develop --release
```

完整的 API 说明、数据布局语义与面向 Agent 的使用指南见
[docs/python-api.md](./docs/python-api.md)。

## License
GPL-3.0 License. See [License here](./LICENSE) for details.
