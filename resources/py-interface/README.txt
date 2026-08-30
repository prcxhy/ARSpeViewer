ARSpeViewer Python 接口（arspe_py）
====================================

本目录在 release 构建时由 CI 填充以下文件（本地源码构建时仅有本说明）：

  arspe_py.pyd / arspe_py.so   Python 扩展模块（abi3，Python 3.10–3.13+ 通用）
  arspe_py.pyi                 类型存根（IDE / 类型检查用）

使用前提：已安装 Python >= 3.10 与 numpy（pip install numpy）。

用法（把本目录加入 sys.path 后导入）：

  import sys
  sys.path.append(r"C:\Program Files\arspeviewer\py-interface")        # Windows
  # macOS: /Applications/arspeviewer.app/Contents/Resources/py-interface
  import arspe_py

  spe = arspe_py.open_file(r"D:\data\run01.spe")
  print(spe.frames.shape)          # (帧数, 角度行数, 波长列数)

完整文档见项目仓库 docs/python-api.md。
