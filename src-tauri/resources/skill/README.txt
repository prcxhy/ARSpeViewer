ARSpeViewer Agent 技能（SKILL.md）
====================================

本目录在 release 构建时由 CI 填充（复制仓库 .agents/skills/arspe-viewer/SKILL.md），
随应用安装到 <安装目录>\skill\SKILL.md。App 内「Agent 接入」按钮会复制一段接入
提示词，粘贴给 AI Agent 即可完成技能安装（提示词会把技能文件中的
<ARSPEVIEWER_INSTALL_DIR> 占位符替换为实际安装目录）。

本地源码构建时本目录仅有本说明（防止 tauri resources glob 报错）。
