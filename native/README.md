# HLG 0.2.0 · 理工恐怖故事

Rust 原生 Windows x64 游戏。开局车卡，使用属性、行动牌、支援手牌和骰子，在固定表世界与反复重组的里世界寻找八件信物并离校。所有图片、中文字体、故事数据及环境音嵌入 EXE。

- 表世界：68 个 TXT 展开节点，保留坐标和地名，执行两项用户批准的勘误。
- 里世界：16 区、1101 个空间节点，区段位置和通路重复生成；原地点与剧情状态保持连续。
- 47 件有实际用途的物品；99 项原稿成就全文，51 项接入触发，48 项未开放。
- 角色成长、可消耗的支援手牌、掷骰结算、逐行动动画、实际差值回执、分页长文阅读。
- 六种原案结局判定；本版完整路线验证覆盖「循环」。

完整范围及未完成项见 [设计与边界](docs/DESIGN_CONTRACT.md) 和 [对照清单](docs/coverage.json)。此包没有把原案全量移植冒称为已经完成。

## 构建

已验证工具链：`rustc 1.99.1 (58cf8f9c3 2026-10-13)`，`stable-x86_64-pc-windows-gnu`。依赖版本固定在 Cargo.lock，macroquad 0.4.16 负责原生窗口与渲染。

在 Windows、安装对应 Rust GNU 工具链后：

```powershell
powershell -ExecutionPolicy Bypass -File tools/Build.ps1 -Task test
powershell -ExecutionPolicy Bypass -File tools/Build.ps1 -Task release
```

程序位于 `target/release/HLG.exe`。Build.ps1 计算构建输入指纹并嵌入程序。直接 `cargo build --release` 也能构建，但如果没有设置指纹环境变量，诊断中会显示 `unrecorded-local-build`，不能冒用随包验收记录。

`native/windows-gnu/libimm32.a` 是随包的 GNU 导入库，说明和上游许可证已附。`--features qa` 仅用于验证构建；分发 EXE 必须关闭该功能。规则测试会把公开行动生成的场景快照写到 `work/qa-states`。原生交互复查可按以下顺序运行（测试须先通过）：

```powershell
powershell -ExecutionPolicy Bypass -File tools/Build.ps1 -Task qa
.\target\release\HLG.exe --qa-output "$PWD\work\qa-render\review"
```

这个验证只向程序自己的 Canvas 输入点击并保存自身帧缓冲，不控制其他应用。生产包重新以 `-Task release` 构建。

## 运行与存档

双击 EXE 即可运行；不需要浏览器、服务器或联网。全部游戏操作在窗口内完成。存档位于 `%LOCALAPPDATA%\HLG-NightSchool-v2`，不会读取或改写 0.1.x 存档。

```powershell
.\HLG.exe --seed 101
.\HLG.exe --diagnose C:\Temp\HLG-diagnostic
```

第一条只固定下一次新循环的种子，不改变既有存档。第二条在独立目录启动，渲染 30 帧后导出启动图与诊断并退出。

## 源码结构

| 文件 | 负责内容 |
|---|---|
| `src/runtime.rs`, `src/frontend.rs` | 原生主循环、输入、界面、动作播放和阅读 |
| `src/world.rs`, `src/echo.rs` | 原稿目录、固定地图、动态里世界 |
| `src/rules.rs`, `src/actions.rs` | 状态、骰子、敌人、资源、死亡与结局 |
| `src/choices.rs`, `src/quests.rs`, `src/cards.rs` | 行动条件、物品配方、事件与手牌 |
| `src/character.rs`, `src/report.rs`, `src/persist.rs` | 车卡成长、差值反馈、存档 |
| `src/story.rs`, `data/*.json` | 叙述、原文、成就、物品和来源依据 |
| `src/rules_tests.rs`, `src/native_qa.rs` | 规则/完整路线与原生界面验证 |
| `tools/verify_conformance.py` | 原始材料逐字/坐标核验及证据身份 gate |

`lib.rs` 显式把 world/rules/report/persist 模块映射为 content/game/feedback/save。没有旧 0.1.x 逻辑参与编译。

原始材料的文件指纹见 `data/authority.json`。完整素材提示、内置 image_gen 使用记录、字体来源和依赖许可见 `docs/ASSET_PROVENANCE.txt` 与 `licenses/`。原稿作者及合作署名以所提供材料为准，本版不将文集中所有篇目归给同一作者。
