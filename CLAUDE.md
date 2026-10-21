# sj-HLG

《理工恐怖故事 / HLG》当前原生 Windows 版：[native/README.md](native/README.md)；此前 HTML 演示：[demo/demo-v2.html](demo/demo-v2.html)。项目知识从 [docs/index.md](docs/index.md) 进入。

- 游戏文本一律照抄[原策划案](docs/design/index.md)，不自行改写或补写；原案是占位文本就保留占位。
- 原生版已有的玩法与叙述改编以 [用户授权记录](native/data/authority.json) 和 [实现边界](native/docs/DESIGN_CONTRACT.md) 为准。

## 文档规则

- 项目知识放在 `docs/`，入口 `docs/index.md`；本文件只写每次会话都需要的内容。
- 分级索引：`docs/index.md` 链接主题索引或笔记；主题超过约 8 条就建自己的 index。新笔记要加到对应索引。
- 一个事实只有一个归属文件，其他地方链接过去，不重复。
- 事实变化时在同一次修改中改掉或删掉旧文字，不只是追加。
- 越常读的文件越短：CLAUDE.md < `docs/index.md` < 主题索引 < 笔记。
- 图片和大文档（docx、pdf 等）不进仓库，挂在 GitHub issue 上，文档里链接 issue。
- `native/assets/` 中的图片、字体和音效是编译所需游戏资源；原始大文档和额外说明插图仍遵守上一条。
- 改完文档运行 `lychee --offline --no-progress CLAUDE.md docs` 并修复断链；去掉 `--offline` 可同时检查外链。

