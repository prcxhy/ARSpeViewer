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

## License
GPL-3.0 License. See [License here](./LICENSE) for details.
