# 原生 Windows 版

当前版本为 **0.2.0**，源码位于 [native/](../../native/README.md)，[下载文件](../../releases/v0.2.0/README.md) 已附 SHA-256 校验值。本次上传采用已经验证的交付文件，未改动游戏实现。

- [游玩说明与攻略](../../native/docs/PLAYER_GUIDE.md)
- [设计、来源与实现边界](../../native/docs/DESIGN_CONTRACT.md)
- [逐项实现清单](../../native/docs/coverage.json)
- [原稿与用户授权记录](../../native/data/authority.json)
- [素材生成记录与来源](../../native/docs/ASSET_PROVENANCE.txt)
- [0.2.0 验证报告](verification/v0.2.0/GATE_REPORT.json)

## 已验证内容

本版支持四属性车卡、角色成长、行动牌和消耗型支援手牌，以骰点与属性结算移动、探索及战斗。表世界采用 68 个 TXT 节点及已批准的坐标勘误。里世界为 16 区、1101 个空间节点，在穿镜、死亡、跨区、周期推进和主动折返时重组相应区域。

发布前通过 19 项规则测试、34 项原生界面检查和 21 项对照检查。六个固定种子通过公开游戏行动完成八信物「循环」路线；测试驱动可以查看图结构，该结果不代表首次游玩的难度已经由真实玩家验证。分发 EXE 在 Windows x64 下完成独立启动检查。

## 尚未完成

整体状态仍为 `PARTIAL_WITH_DECLARED_GAPS`。99 项原稿成就全文已保留，其中 51 项接入触发，48 项未开放；成就界面为分页列表，尚未实现完整节点树。其他五种结局有判定测试，未完成从开局到结局的全流程验证。原稿的全部剧情、可选物品用法和支线尚未完整移植。

本目录描述 Rust 原生版；[HTML 演示进度](../demo/status.md) 继续描述原有演示。原策划文档保持在 [design/](../design/index.md)。
