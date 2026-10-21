use crate::{cards::*, content::*, game::*};
impl Game {
    pub fn move_cost(&self, mode: TravelMode) -> i32 {
        (match mode {
            TravelMode::Elevator => 0,
            TravelMode::Slip | TravelMode::Leap | TravelMode::Fly => 1,
            _ => {
                (if self.flag("encountered") { 1 } else { 0 })
                    + (if mode == TravelMode::Careful { 1 } else { 0 })
            }
        }) + (if self.in_combat() { 1 } else { 0 })
            + (if self.equip == Some(Item::Diploid) {
                1
            } else {
                0
            })
            + (if self.counter("cold_until") > self.turn && self.equip != Some(Item::Wenjin) {
                1
            } else {
                0
            })
    }
    pub fn move_modes(&self, to: usize) -> Vec<TravelMode> {
        if to >= NODES.len() || to == self.current || !NODES[to].exists(self.world) {
            return vec![];
        }
        let here = &NODES[self.current];
        let there = &NODES[to];
        let mut modes = vec![];
        let a = self.position(self.current);
        let b = self.position(to);
        if self.neighbors(self.current).contains(&to) {
            modes.extend([TravelMode::Walk, TravelMode::Careful]);
        }
        if self.equip == Some(Item::Soap)
            && self
                .neighbors(self.current)
                .iter()
                .any(|&n| self.entry_reason(n).is_none() && self.neighbors(n).contains(&to))
        {
            modes.push(TravelMode::Slip);
        }
        if self.equip == Some(Item::Thunder)
            && b[2] > a[2]
            && a[0] == b[0]
            && a[1] == b[1]
            && !there.interior
        {
            modes.push(TravelMode::Leap);
        }
        if self.equip == Some(Item::Wheel) && here.roof() && there.roof() {
            modes.push(TravelMode::Fly);
        }
        if here.name == "综合楼电梯" && there.name == "综合楼电梯" {
            modes.push(TravelMode::Elevator);
        }
        modes
    }
    pub fn entry_reason(&self, to: usize) -> Option<String> {
        let n = &NODES[to];
        match n.id.as_str() {
            "classroom" if !self.has(Item::Card) => {
                Some("需要携带波波的门卡。前往三层化学办公室。".into())
            }
            "storage" | "chem_inner" if !self.flag("met_wxh") => {
                Some("须先触发 WXH 剧情；门卡就在化学办公室。".into())
            }
            "saponify" if !self.flag("soap_story") => {
                Some("先用粗制肥皂驱离 WXH，听完他对成品的要求。".into())
            }
            "captive" => Some("豢养区的专属剧情入口在本章尚未开放。".into()),
            "principal" if RELICS.iter().take(7).any(|&i| !self.has(i)) => {
                Some("门前的题词要求其余七件信物。".into())
            }
            _ => None,
        }
    }
    pub fn movement_cards(&self, to: usize) -> Vec<Card> {
        self.move_modes(to)
            .into_iter()
            .map(|mode| {
                let title = match mode {
                    TravelMode::Walk => "循路前行",
                    TravelMode::Careful => "贴墙慢行",
                    TravelMode::Slip => "香皂滑步",
                    TravelMode::Leap => "怒霆上跃",
                    TravelMode::Fly => "骨轮飞渡",
                    TravelMode::Elevator => "乘坐电梯",
                };
                let cost = self.move_cost(mode);
                let target = if self.in_combat() {
                    8
                } else if self.world == World::Echo {
                    7
                } else {
                    5
                };
                let rule = match mode {
                    TravelMode::Elevator => {
                        "到达所选楼层；d6 = 1 时出现黑洞，承受 12 生命伤害。".into()
                    }
                    TravelMode::Slip => "沿两段实际通路抵达；不用移动检定。".into(),
                    TravelMode::Leap => "沿当前坐标向上抵达所选楼层；不用移动检定。".into(),
                    TravelMode::Fly => "跨越至目标楼顶；消耗 1 燃油，不用移动检定。".into(),
                    _ => format!(
                        "d6 + 机敏{} ≥ {target}；未通过仍到达，遭遇后多耗 1 体力。",
                        if mode == TravelMode::Careful {
                            " + 2"
                        } else {
                            ""
                        }
                    ),
                };
                let reason = self.entry_reason(to);
                let mut c = Card::new(
                    Action::Travel { to, mode },
                    title,
                    &format!(
                        "前往{} · {}层。先核对脚下的路，再决定要不要让脚步被走廊听见。",
                        NODES[to].name,
                        self.position(to)[2]
                    ),
                    &rule,
                    &format!(
                        "{cost} 体力{}；推进一回合",
                        if mode == TravelMode::Fly {
                            " + 1 燃油"
                        } else {
                            ""
                        }
                    ),
                    "TXT:移动系统;U09/U13",
                );
                c = c
                    .require(reason.is_none(), reason.unwrap_or_default())
                    .require(
                        mode != TravelMode::Fly || self.has(Item::Fuel),
                        "骨轮飞行需要 1 份燃油。",
                    );
                c.icon = match mode {
                    TravelMode::Slip => Some(Item::Soap),
                    TravelMode::Leap => Some(Item::Thunder),
                    TravelMode::Fly => Some(Item::Wheel),
                    _ => None,
                };
                c
            })
            .collect()
    }
    pub fn cards(&self, tab: CardTab, target: Option<usize>) -> Vec<Card> {
        match tab {
            CardTab::Move => target.map(|n| self.movement_cards(n)).unwrap_or_default(),
            CardTab::Explore => {
                let mut v = self.quest_cards();
                v.push(
                    Card::new(
                        Action::Explore(Approach::Search),
                        "逐处查看",
                        "把灯照到桌角与柜缝里，检查那些白天不会特意留意的地方。",
                        if self.world == World::Echo {
                            "d6 + 洞察 ≥ 7；成功核实本区地图并准备探索 +2，失败损失 2 理智。"
                        } else {
                            "d6 + 洞察 ≥ 7；成功准备探索 +2，失败损失 2 理智。"
                        },
                        "1 体力；一回合",
                        "U08,U09,U13",
                    )
                    .require(
                        self.world == World::Echo || !self.flag("prepared"),
                        "已经充分查看：下一次探索检定 +2。",
                    ),
                );
                v.push(Card::new(
                    Action::Explore(Approach::Study),
                    "对读记录",
                    "将刚记下的内容与眼前的地点核对，看看是谁在替谁解释这所学校。",
                    "d6 + 洞察 ≥ 7；成功恢复 5 理智，失败无恢复。",
                    "1 体力；一回合",
                    "U09",
                ));
                v.push(
                    Card::new(
                        Action::Shift,
                        "走入倒影",
                        "你把手伸向反光的那一面，等指尖碰到并不存在的门槛，再慢慢跨过去。",
                        "d6 + 定力 ≥ 7；两种结果都会换面，未通过额外损失 3 理智。",
                        if self.world == World::Surface {
                            "1 体力、12 理智；一回合"
                        } else {
                            "1 体力、恢复 6 理智；一回合"
                        },
                        "U02,U09",
                    )
                    .require(
                        self.flag("mirror") && NODES[self.current].mirror,
                        "先在喷泉确认倒影，并来到镜面地点。",
                    )
                    .require(
                        self.world == World::Echo || self.sanity > 15,
                        "进入里世界至少需要 16 理智。",
                    ),
                );
                v.push(
                    Card::new(
                        Action::Rest,
                        "坐下休整",
                        "你挑一处能看见出口的位置坐下，等呼吸和走廊的杂音慢慢分开。",
                        "体力补满；安全地点恢复 8 理智与 4 生命，其他地点只恢复体力。",
                        "一回合；无检定",
                        "TXT:移动系统",
                    )
                    .require(!self.in_combat(), "有实体在场，先驱离或离开。"),
                );
                if self.world == World::Echo {
                    v.push(Card::new(Action::Reshape,"沿走廊折返","身后的楼梯还是刚才那一道，门牌却已经不在原处。你往回走了几步，等走廊尽头的灯逐一亮起，才重新确认自己站在哪一边。","本区房间与通路重新排列，旧地图失效；剧情与背包保留。d6 + 定力 ≥ 7：损失 1 理智，否则损失 4。","1 体力；一回合","U13/U09").require(!self.in_combat(),"先离开当前实体，才有余裕尝试折返。"));
                }
                if self.at("clock") {
                    v.push(Card::new(
                        Action::SaveMail,
                        "寄存这一刻",
                        "将背包清单折好，和刚才写下的日期一起送进邮筒。",
                        "覆盖邮筒寄存清单；下次死亡返回钟楼并恢复这份物品清单。",
                        "一回合；无检定",
                        "TXT:死亡与存档",
                    ));
                }
                v
            }
            CardTab::Combat => {
                let enemy = self.active_enemies().first().map(|&i| self.enemies[i].kind);
                let present = enemy.is_some();
                let mut v = vec![
                    Card::new(
                        Action::Attack(AttackMode::Strike),
                        "迎击",
                        "握稳手中的东西，等它靠近时再出手。",
                        "攻击 d(4 + 体魄 + 器物加成) 对敌方 d4；正差值为伤害。",
                        "1 体力；敌人随后行动",
                        "TXT:战斗系统;U09",
                    )
                    .require(present, "当前没有正在交战的实体。"),
                    Card::new(
                        Action::Defend,
                        "稳住架势",
                        "不急着向前，先把身体藏在课桌或墙角能遮住的地方。",
                        "本回合防御骰额外增加 4 面；下一次攻击不继承此加值。",
                        "0 体力；敌人行动一次",
                        "U09",
                    )
                    .require(present, "当前没有正在交战的实体。"),
                ];
                for (i, mode, title, rule, cost) in [
                    (
                        Item::Lamp,
                        AttackMode::Lamp,
                        "灯火驱离",
                        "攻击后将 WXH 沿来路击退一格；d2 = 2 时解除皂化。",
                        "1 体力",
                    ),
                    (
                        Item::Stroke,
                        AttackMode::Stroke,
                        "一横发射",
                        "d10 对 d4，造成正差伤害并击退；需直线目标。",
                        "1 体力 + 1 黏痰",
                    ),
                    (
                        Item::Diploid,
                        AttackMode::Assimilate,
                        "同化压制",
                        "d(6 + 体魄) 对 d4；只对复制体和被同化者生效。",
                        "1 体力",
                    ),
                    (
                        Item::Domain,
                        AttackMode::Domain,
                        "黑域环绕",
                        "d(6 + 定力) 对 d4；吸引附近实体并攻击。",
                        "1 体力",
                    ),
                    (
                        Item::Thunder,
                        AttackMode::Thunder,
                        "天雷庇佑",
                        "消耗一次充能，将在场实体驱离一格并暂停两回合；累计四次会改变结局。",
                        "1 体力 + 1 充能",
                    ),
                ] {
                    if self.has(i) {
                        let mut c = Card::new(
                            Action::Attack(mode),
                            title,
                            i.description(),
                            rule,
                            cost,
                            &i.info().source,
                        )
                        .item(i)
                        .require(
                            self.combat_target(mode).is_some(),
                            "当前没有符合范围的交战目标。",
                        )
                        .require(
                            self.equip == Some(i) || mode == AttackMode::Thunder,
                            "需要先持握这件物品。",
                        );
                        if mode == AttackMode::Lamp {
                            c = c
                                .require(enemy == Some(EnemyKind::Wxh), "酒精灯的驱离只针对 WXH。");
                        }
                        if mode == AttackMode::Assimilate {
                            c = c.require(
                                enemy.is_some_and(|e| e.mortal()),
                                "同化攻击只影响复制体与被同化者。",
                            );
                        }
                        if mode == AttackMode::Stroke {
                            c = c.require(self.has(Item::Slug), "需要 1 份黏痰作为弹药。");
                        }
                        if mode == AttackMode::Thunder {
                            c = c.require(
                                self.has(Item::ThunderSpear) && self.counter("charge") > 0,
                                "需要经过 ZYY 改造的钧霆雷枪及一次雷跃充能。",
                            );
                        }
                        v.push(c);
                    }
                }
                for &i in self.inv.keys() {
                    if matches!(
                        i,
                        Item::Water | Item::Food | Item::Milk | Item::RoughSoap | Item::Fusion
                    ) {
                        v.extend(self.item_cards(i));
                    }
                }
                v
            }
            CardTab::Items => {
                let mut v = vec![];
                for &i in self.inv.keys() {
                    v.extend(self.item_cards(i));
                }
                v.extend(self.craft_cards());
                if self.equip.is_some() {
                    v.push(Card::new(
                        Action::Equip(None),
                        "腾出双手",
                        "将持握物收回书包，确认拉链已经拉好。",
                        "停止其持握效果；可空手取出标本。",
                        "不推进回合",
                        "TXT:物品系统",
                    ));
                }
                v
            }
        }
    }
    pub fn item_cards(&self, i: Item) -> Vec<Card> {
        let mut v = vec![];
        if i.equippable() {
            v.push(
                Card::new(
                    Action::Equip(Some(i)),
                    &format!("持握 · {}", i.name()),
                    i.description(),
                    i.effect(),
                    "不推进回合",
                    &i.info().source,
                )
                .item(i)
                .require(self.equip != Some(i), "已经持握。"),
            );
        }
        let active = matches!(
            i,
            Item::Water
                | Item::Food
                | Item::Milk
                | Item::RoughSoap
                | Item::Fusion
                | Item::Meter
                | Item::Record
                | Item::Void
                | Item::DarkToken
                | Item::Hook
                | Item::Coal
                | Item::Ore
                | Item::EmptyBottle
                | Item::Key
        );
        if active {
            let mut c = Card::new(
                Action::Use(i),
                &format!("使用 · {}", i.name()),
                i.description(),
                i.effect(),
                "一回合；所需材料在结算前检查",
                &i.info().source,
            )
            .item(i);
            let reason = self.use_reason(i);
            c = c.require(reason.is_none(), reason.unwrap_or_default());
            v.push(c);
        }
        // Materials lead to a real recipe card, carried permissions to the
        // contextual quest card. Inventory detail lists these without consuming it.
        for &(out, parts) in RECIPES {
            if parts.iter().any(|(p, _)| *p == i) || i == Item::Mold && out == Item::RefinedSoap {
                if let Some(c) = self
                    .craft_cards()
                    .into_iter()
                    .find(|c| c.action == Action::Craft(out))
                {
                    v.push(c);
                }
            }
        }
        if self.at("canteen") && !i.retained() && !matches!(i, Item::Water) {
            v.push(
                Card::new(
                    Action::Barter(i),
                    "向水站投物",
                    "你先将物品放入投物盒，再去取旁边的矿泉水。",
                    "交出 1 件所选物品，取得 1 矿泉水；诚信水站继续供水。",
                    "所选物品 ×1；一回合",
                    "MC:223-229",
                )
                .item(i),
            );
        }
        v
    }
    pub fn craft_cards(&self) -> Vec<Card> {
        RECIPES
            .iter()
            .map(|&(out, parts)| {
                let desc = parts
                    .iter()
                    .map(|(i, n)| format!("{} ×{}", i.name(), n))
                    .collect::<Vec<_>>()
                    .join(" + ");
                Card::new(
                    Action::Craft(out),
                    &format!("制作 · {}", out.name()),
                    out.description(),
                    &format!(
                        "{desc} → {}。配方正确时确定完成；材料在成功前不扣除。",
                        out.name()
                    ),
                    "一回合；消耗配方材料",
                    &out.info().source,
                )
                .item(out)
                .require(
                    parts.iter().all(|&(i, n)| self.count(i) >= n),
                    format!("材料不足：{desc}"),
                )
                .require(
                    out != Item::RefinedSoap || self.has(Item::Mold),
                    "需要制皂模具；模具不消耗。",
                )
            })
            .collect()
    }
    pub fn use_reason(&self, i: Item) -> Option<String> {
        if !self.has(i) {
            return Some("背包中没有这件物品。".into());
        }
        let ok = match i {
            Item::Coal | Item::Ore => self.at("geo_office"),
            Item::Hook => self.at("dungeon") || self.rescue_key().is_some(),
            Item::EmptyBottle => self.at("storage"),
            Item::Key => self.at("lockers") && !self.flag("hidden_cabinet"),
            Item::Meter => {
                self.has(Item::Thunder) && (self.at("physics_lab") || self.at("electric_lab"))
            }
            Item::RoughSoap => self
                .active_enemies()
                .iter()
                .any(|&j| self.enemies[j].kind == EnemyKind::Wxh),
            Item::Fusion => {
                self.active_enemies()
                    .iter()
                    .filter(|&&j| self.enemies[j].kind == EnemyKind::Copy)
                    .count()
                    >= 2
            }
            Item::Record => !self.flag("used_record") && !self.flag("used_void"),
            Item::Void => !self.flag("used_void") && !self.flag("used_record"),
            Item::DarkToken => !self.flag("dark_active"),
            Item::Leaf => self.equip != Some(Item::Leaf),
            Item::Water | Item::Food | Item::Milk => true,
            _ => false,
        };
        if ok {
            None
        } else {
            Some(
                match i {
                    Item::Coal | Item::Ore => "前往地理办公室，向高炉投料。",
                    Item::Hook => "需要地牢水池或尚未救出的学长。",
                    Item::EmptyBottle => "在化学储藏间领料口使用。",
                    Item::Key => "只用于尚未开启的钟楼隐藏柜。",
                    Item::Meter => "需要携带雷跃，来到物理/电学实验室承受雷击充能。",
                    Item::RoughSoap => "需要正在交战的 WXH。",
                    Item::Fusion => "至少两个复制体在场才可融合。",
                    Item::Record | Item::Void => "已经作出结局选择，不能撤销或同时启用另一条路线。",
                    Item::DarkToken => "已经启用深渊信物。",
                    Item::Leaf => "已经持握金色槐叶。",
                    _ => "这是携带条件或制作材料，请查看对应事件与配方。",
                }
                .into(),
            )
        }
    }
    pub fn legal_card(&self, a: &Action) -> Option<Card> {
        if let Action::Train(attr) = a {
            return Some(
                Card::new(
                    a.clone(),
                    "积累成经验",
                    "把这一次经历记在心里，下次或许能多撑片刻。",
                    attr.meaning(),
                    "成长点 ×1；不推进回合",
                    "U09",
                )
                .require(
                    self.character.growth > 0 && self.character.stat(*attr) < 8,
                    "没有成长点，或该属性已达到 8。",
                ),
            );
        }
        let target = match a {
            Action::Travel { to, .. } => Some(*to),
            _ => None,
        };
        [
            CardTab::Move,
            CardTab::Explore,
            CardTab::Combat,
            CardTab::Items,
        ]
        .into_iter()
        .flat_map(|t| self.cards(t, target))
        .find(|c| &c.action == a)
    }
}
