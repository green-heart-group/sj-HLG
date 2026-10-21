use crate::{content::*, game::*};
#[derive(Debug, Clone)]
pub struct Receipt {
    pub title: String,
    pub story: String,
    pub changes: Vec<String>,
    pub notes: Vec<String>,
    pub rolls: Vec<Roll>,
    pub major: bool,
    pub accepted: bool,
}
impl Receipt {
    pub fn arrival(g: &Game) -> Self {
        Self {
            title: g.last.title.clone(),
            story: g.last.story.clone(),
            changes: if g.turn == 0 {
                vec!["起始物品：矿泉水 ×1、威化饼干 ×2。".into()]
            } else if !g.last.changes.is_empty() {
                g.last.changes.clone()
            } else {
                vec![format!("已恢复第 {} 回合的记录。", g.turn)]
            },
            notes: g.last.notes.clone(),
            rolls: g.last.rolls.clone(),
            major: true,
            accepted: true,
        }
    }
    pub fn capture(before: &Game, after: &Game) -> Self {
        let mut changes = vec![];
        for (j, e) in after.enemies.iter().enumerate() {
            if let Some(old) = before.enemies.get(j) {
                if old.node != e.node && e.hp > 0 {
                    changes.push(format!(
                        "{} 已移至{}{}",
                        e.kind.name(),
                        NODES[e.node].name,
                        if e.stun > old.stun {
                            format!("（停攻 {} 回合）", e.stun)
                        } else {
                            String::new()
                        }
                    ));
                } else if e.stun > old.stun && e.hp > 0 {
                    changes.push(format!("{} 停攻 {} 回合", e.kind.name(), e.stun));
                }
                if e.hp != old.hp {
                    changes.push(format!(
                        "{} 生命 {} → {}{}",
                        e.kind.name(),
                        old.hp.max(0),
                        e.hp.max(0),
                        if e.hp <= 0 {
                            "（失去行动能力）"
                        } else {
                            ""
                        }
                    ));
                }
            } else if e.hp > 0 {
                changes.push(format!("新遭遇：{}", e.kind.name()));
            }
        }
        if before.flag("saponified") != after.flag("saponified") {
            changes.push(if after.flag("saponified") {
                "新增状态：皂化，每回合额外耗费 1 体力。".into()
            } else {
                "皂化状态已解除。".into()
            });
        }
        for (flag, text) in [
            ("guide", "已确认离校要求：带齐八件信物向传达室交付。"),
            ("mirror", "已确认镜面入口：可以穿行表里世界。"),
            ("soap_story", "皂化池事件已开启。"),
            ("gate_open", "校门已开启，可以前往离校。"),
            ("used_record", "你的手记已启用：探索 +1，离校条件改变。"),
            (
                "used_void",
                "虚无的造物已启用：持续恢复生命，离校条件改变。",
            ),
            ("dark_active", "深渊信物已启用：实体停攻，每回合理智 -2。"),
            ("prepared", "已经准备：下一次适用的探索检定 +2。"),
        ] {
            if !before.flag(flag) && after.flag(flag) {
                changes.push(text.into());
            }
        }
        for (name, b, a) in [
            ("生命", before.hp, after.hp),
            ("体力", before.stamina, after.stamina),
            ("理智", before.sanity, after.sanity),
        ] {
            if a != b {
                changes.push(format!("{name} {b} → {a}（{:+}）", a - b));
            }
        }
        for (name, b, a) in [
            (
                "生命上限",
                before.character.max_hp(),
                after.character.max_hp(),
            ),
            ("体力上限", before.max_stamina(), after.max_stamina()),
        ] {
            if a != b {
                changes.push(format!("{name} {b} → {a}（{:+}）", a - b));
            }
        }
        if before.current != after.current || before.world != after.world {
            changes.push(format!(
                "地点：{}·{} → {}·{}",
                NODES[before.current].name,
                before.world.name(),
                NODES[after.current].name,
                after.world.name()
            ));
        }
        if before.echo.revision != after.echo.revision {
            changes.insert(
                0,
                format!(
                    "里世界地图重组：第 {} → {} 次。旧通路已经失效，重新查看本区地图。",
                    before.echo.revision, after.echo.revision
                ),
            );
        }
        for &i in ALL_ITEMS {
            let b = before.count(i) as i32;
            let a = after.count(i) as i32;
            if a != b {
                changes.push(format!("{} {:+}（现有 {}）", i.name(), a - b, a));
            }
        }
        if before.equip != after.equip {
            changes.push(format!(
                "持握：{}",
                after.equip.map_or("双手空闲", |i| i.name())
            ));
        }
        if before.hand != after.hand {
            changes.push(format!(
                "当前手牌：{}",
                after
                    .hand
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(" / ")
            ));
        }
        if before.character.xp != after.character.xp {
            changes.push(format!(
                "阅历 +{}；等级 {}；成长点 {}",
                after.character.xp - before.character.xp,
                after.character.level,
                after.character.growth
            ));
        }
        for (i, &v) in after.character.values.iter().enumerate() {
            if v != before.character.values[i] {
                changes.push(format!(
                    "{} {} → {}",
                    crate::character::ATTRIBUTES[i].name(),
                    before.character.values[i],
                    v
                ));
            }
        }
        for id in after.clues.difference(&before.clues) {
            if let Some(d) = DOCUMENTS.iter().find(|d| &d.id == id) {
                changes.push(format!("新手记：{}", d.title));
            }
        }
        if changes.is_empty() {
            changes.push("物品、属性与位置没有变化。".into());
        }
        changes.push(if after.turn == before.turn {
            "不推进回合。".into()
        } else {
            format!("第 {} 回合 → 第 {} 回合。", before.turn, after.turn)
        });
        Self {
            title: after.last.title.clone(),
            story: after.last.story.clone(),
            changes,
            notes: after.last.notes.clone(),
            rolls: after.last.rolls.clone(),
            major: after.last.major,
            accepted: after.last.accepted,
        }
    }
}
#[derive(Clone)]
pub struct Guide {
    pub title: String,
    pub detail: String,
    pub destination: Option<usize>,
    pub world: Option<World>,
}
pub fn guide(g: &Game) -> Guide {
    let (title, detail, key, world) = if g.in_combat() {
        (
            "先处理眼前的实体",
            "查看它的攻击骰和生命/精神目标。选择战斗卡，或点地图上的相邻地点，用移动卡离开。",
            None,
            None,
        )
    } else if g.flag("saponified") && g.has(Item::Milk) {
        (
            "用牛奶解除皂化",
            "皂化每回合额外扣 1 体力。打开背包选牛奶，或在道具牌中使用，随后再继续探索。",
            None,
            None,
        )
    } else if g.character.growth > 0 {
        (
            "有可分配的成长点",
            "打开人物卡，将阅历转为属性。提高体魄会同时增加当前生命和生命上限。",
            None,
            None,
        )
    } else if g.stamina < 3 {
        (
            "先恢复体力",
            "无实体在场时可以休整补满体力；安全点还会恢复生命和理智。",
            None,
            None,
        )
    } else if g.sanity < 30 {
        (
            "先稳住神智",
            "使用「对读记录」恢复理智，或到学生公司休整。里世界每个区段均有休整处和返回镜面。",
            None,
            None,
        )
    } else if !g.flag("guide") {
        (
            "先读离校须知",
            "传达室在校门西侧，先去确认八件信物应当交到哪里。",
            Some("reception"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Lamp) {
        if [Item::Ink, Item::Matches, Item::Solution]
            .iter()
            .all(|&i| g.has(i))
        {
            (
                "把三件材料组装成灯",
                "在背包或物品卡组中选择「制作·自制酒精灯」。之后可持握它驱离 WXH。",
                None,
                None,
            )
        } else {
            (
                "寻找学长留下的材料",
                "去钟楼柜群打开柜子，三件材料和受潮手记在那里。",
                Some("lockers"),
                Some(World::Surface),
            )
        }
    } else if !g.has(Item::Card) {
        (
            "到三层化学办公室取门卡",
            "先持握酒精灯。取卡会触发传送和遭遇；留好应对手段。",
            Some("chem_office"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Wenjin) {
        (
            "回高三一班拿作业",
            "教室在四层南走廊附近。门卡只需携带，取得问津后可用来应对 LL。",
            Some("classroom"),
            Some(World::Surface),
        )
    } else if !g.flag("mirror") {
        (
            "确认水面另一侧的入口",
            "回喷泉，选择等待倒影的事件卡。镜面地点随后可使用穿镜卡。",
            Some("fountain"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Hook) {
        (
            "去园林取引魂钓",
            "钓竿能打捞地牢中的器物，也能在后续事件里救人。",
            Some("garden"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Stroke) {
        (
            "打捞假山地牢的器物",
            "到地下地牢，使用引魂钓；失败不会消耗钓竿，可以重试。",
            Some("dungeon"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Thunder) {
        (
            "找实验楼里的雷具",
            "物理实验室在二层；箱内有雷跃与笔记，取出后会遭遇 ZYY。",
            Some("physics_lab"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Diploid) {
        (
            "查明标本馆里的样本",
            "在里世界进入生物实验室内的标本馆；取标本前先腾出双手。",
            Some("specimen"),
            Some(World::Echo),
        )
    } else if !g.has(Item::Boots) {
        (
            "点亮地理办公室的高炉",
            "领取矿料与三块煤炭，先后投进高炉；三座炉火一并点燃后取得石靴和钥匙。",
            Some("geo_office"),
            Some(World::Surface),
        )
    } else if !g.has(Item::Wheel) {
        (
            "前往综合楼顶层的内间",
            "电梯可达各层；顶层里世界的炼功房存放骨轮。",
            Some("training"),
            Some(World::Echo),
        )
    } else if !g.has(Item::Soap) {
        if !g.flag("soap_story") {
            if !g.has(Item::RoughSoap) && g.has(Item::Fat) && g.has(Item::Alkali) {
                (
                    "配制粗制肥皂",
                    "用脂肪与不明溶液制作粗制肥皂，再到化学实验室投向 WXH。",
                    Some("chem_lab"),
                    Some(World::Surface),
                )
            } else if !g.has(Item::Fat) && !g.has(Item::RoughSoap) {
                ("取得制皂材料","里世界课程主任办公室有被同化者；战胜后取得脂肪。不明溶液可在化学办公间或储藏间领取。",Some("director"),Some(World::Echo))
            } else {
                (
                    "用粗制肥皂驱离主任",
                    "在与 WXH 交战时使用粗制肥皂，开启皂化池剧情；仍缺材料时到储藏间领取。",
                    Some("chem_lab"),
                    Some(World::Surface),
                )
            }
        } else {
            ("完成皂化池的实验","从里世界化学办公间进入皂化池。原案的细研香皂成品判定为 d7 = 7；死亡保留剧情和信物。",Some("saponify"),Some(World::Echo))
        }
    } else if !g.has(Item::Domain) {
        (
            "带七件信物叩门",
            "回到表世界校长办公室，通过定力检定取得最后一件信物。",
            Some("principal"),
            Some(World::Surface),
        )
    } else if !g.flag("gate_open") {
        ("在传达室交付八信物","第一次交付会触发喷泉剧情，回到大厅；再去交付一次才能开门。离校前也可继续探索结局物品。",Some("reception"),Some(World::Surface))
    } else {
        (
            "校门已经打开",
            "去校门口确认离校。若没有启用任何结局物品，原案判定为「循环」。",
            Some("gate"),
            Some(World::Surface),
        )
    };
    let mut detail = detail.to_owned();
    let destination = key.map(node);
    if g.world == World::Echo {
        if world == Some(World::Surface) {
            detail = format!("先寻找本区喷泉镜面返回表世界。{detail}");
        } else if let Some(n) = destination {
            detail=format!("目标在第 {} 区段。用「逐处查看」核实本区通路；跨区和每 24 个里世界回合会重组地图。",g.echo.zone(n)+1);
        }
    }
    Guide {
        title: title.into(),
        detail,
        destination,
        world,
    }
}
