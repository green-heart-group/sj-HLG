use crate::{cards::*, character::*, content::*, game::*};
impl Game {
    pub fn play(&mut self, a: Action, tactic: Option<Tactic>) -> bool {
        if !self.playing() {
            return false;
        }
        let before_play = self.clone();
        let allowed = self.legal_card(&a).is_some_and(|c| c.enabled);
        if !allowed {
            return self.apply(a);
        }
        if let Some(t) = tactic {
            let Some(i) = self.hand.iter().position(|&x| x == t) else {
                self.last = Resolution::default();
                self.note("这张支援牌不在当前手牌中，行动没有执行。");
                return false;
            };
            if matches!(a, Action::Equip(_) | Action::Train(_)) {
                self.last = Resolution::default();
                self.note("整理装备和成长无需使用支援牌。");
                return false;
            }
            self.hand.remove(i);
            self.discard_pile.push(t);
            self.committed = Some(t);
        }
        let accepted = self.apply(a);
        if let Some(t) = self.committed.take() {
            self.note(format!(
                "支援牌「{}」已弃置，效果：{}。已从牌库补回一张。",
                t.name(),
                t.effect()
            ));
        }
        self.refill_hand();
        self.last.changes = crate::feedback::Receipt::capture(&before_play, self).changes;
        accepted
    }
    pub fn apply(&mut self, a: Action) -> bool {
        if !self.playing() {
            return false;
        }
        let before = self.clone();
        self.last = Resolution::default();
        let Some(card) = self.legal_card(&a) else {
            self.note("此刻没有这张行动卡，状态没有变化。");
            return false;
        };
        self.last.title = card.title;
        self.last.story = card.story;
        if !card.enabled {
            self.note(card.reason);
            return false;
        }
        self.last.accepted = true;
        self.last.motion = match &a {
            Action::Travel { .. } => "walk",
            Action::Attack(_) => "combat",
            Action::Defend => "guard",
            Action::Shift => "mirror",
            Action::Craft(_) => "craft",
            Action::Equip(_) => "equip",
            Action::Use(_) => "item",
            Action::Train(_) => "growth",
            Action::Reshape => "rearrange",
            _ => "explore",
        }
        .into();
        let mut advance = true;
        let mut guard = false;
        match a {
            Action::Travel { to, mode } => self.travel(to, mode),
            Action::Explore(approach) => {
                self.spend(1);
                match approach {
                    Approach::Search => {
                        if self.test("查看环境", Attribute::Insight, 7, 0) {
                            self.mark("prepared");
                            self.note("细节已核对，下一次探索或实验检定 +2。");
                            if self.world == World::Echo {
                                self.survey_echo();
                            }
                        } else {
                            self.sanity -= 2;
                            self.note("没有确认异常的来源；理智 -2，可以改用其他行动或重试。");
                        }
                    }
                    Approach::Study => {
                        if self.test("核对记录", Attribute::Insight, 7, 0) {
                            self.sanity = (self.sanity + 5).min(100);
                            self.note("至少这一段仍与记录相符，理智恢复 5。");
                        } else {
                            self.note("记录之间仍有矛盾，这次没有恢复理智。");
                        }
                    }
                    Approach::Careful => {
                        guard = true;
                        self.note("你记住了它的动作，本回合防御骰增加 4 面。");
                    }
                }
            }
            Action::Quest(id) => self.quest(&id),
            Action::Shift => {
                self.spend(1);
                let entering = self.world == World::Surface;
                if !self.test("越过镜面", Attribute::Will, 7, 0) {
                    self.sanity -= 3;
                }
                self.sanity = if entering {
                    self.sanity - 12
                } else {
                    (self.sanity + 6).min(100)
                };
                if entering {
                    self.entry_surface = self.current;
                    self.reconfigure_echo(None, "镜面另一侧重新展开");
                } else if !NODES[self.current].exists(World::Surface) {
                    self.current = self.entry_surface;
                }
                self.world = self.world.other();
                self.mark("shifted");
                self.last.major = true;
                self.note(format!(
                    "已来到{}。表世界坐标固定，里世界使用本次重组后的坐标与通路。",
                    self.world.name()
                ));
            }
            Action::Reshape => {
                self.spend(1);
                let passed = self.test("辨认折返后的门牌", Attribute::Will, 7, 0);
                self.sanity -= if passed { 1 } else { 4 };
                self.reconfigure_echo(Some(self.echo.zone(self.current)), "主动折返");
            }
            Action::Rest => {
                self.stamina = self.max_stamina();
                if NODES[self.current].safe {
                    self.sanity = (self.sanity + 8).min(100);
                    self.hp = (self.hp + 4).min(self.character.max_hp());
                }
                self.note("休整完成。体力已经补满；恢复量以回执中的实际差值为准。");
            }
            Action::SaveMail => {
                self.mail = Some(self.inv.clone());
                self.note(
                    "邮筒已经收下本次物品清单；再次寄存会替换它，退出游戏的自动保存不受影响。",
                );
            }
            Action::Equip(i) => {
                self.equip = i;
                advance = false;
                if i == Some(Item::Diploid) {
                    self.mark("wore_diploid");
                }
                self.stamina = self.stamina.min(self.max_stamina());
                self.note(
                    i.map(|i| format!("已持握{}。{}", i.name(), i.effect()))
                        .unwrap_or_else(|| "已腾出双手；背包物品数量没有改变。".into()),
                );
            }
            Action::Use(i) => self.use_item(i),
            Action::Craft(i) => {
                for &(p, n) in recipe(i).expect("validated recipe") {
                    self.take(p, n);
                }
                self.give(i, 1);
                self.last.major = i.is_relic() || i.info().kind == "结局";
                self.note(format!(
                    "配方核对完成，取得{}。所有材料均已按配方扣除。",
                    i.name()
                ));
                match i {
                    Item::Lamp => self.unlock("手工艺人"),
                    Item::RoughSoap => self.unlock("皂化反应"),
                    Item::RefinedSoap => self.unlock("精制肥皂"),
                    Item::Meter => self.unlock("动手组装"),
                    Item::Record => self.unlock("你的手记"),
                    Item::DarkToken => {
                        self.mark("made_dark");
                        self.unlock("深渊的信物");
                    }
                    _ => {}
                }
                self.gain_once(&format!("craft_{i:?}"), 2);
            }
            Action::Attack(mode) => self.attack(mode),
            Action::Defend => {
                guard = true;
                self.note("架势已稳住；本回合防御骰增加 4 面，实体将行动一次。");
            }
            Action::Barter(i) => {
                self.take(i, 1);
                self.give(Item::Water, 1);
                self.unlock("诚实守信");
                self.note(format!(
                    "已先投入{}，再领取矿泉水。水站仍可继续使用。",
                    i.name()
                ));
            }
            Action::Train(attr) => {
                let old = self.character.max_hp();
                self.character.train(attr);
                self.hp += self.character.max_hp() - old;
                advance = false;
                self.note(format!(
                    "{}提高 1，剩余成长点 {}。",
                    attr.name(),
                    self.character.growth
                ));
            }
        }
        if advance && self.playing() {
            self.tick(guard);
        }
        if self.playing() && (self.hp <= 0 || self.sanity <= 0) {
            self.rebirth("身体或神智已经无法支撑", false);
        }
        self.hp = self.hp.clamp(1, self.character.max_hp());
        self.sanity = self.sanity.clamp(0, 100);
        self.stamina = self.stamina.clamp(0, self.max_stamina());
        self.reveal();
        self.last.changes = crate::feedback::Receipt::capture(&before, self).changes;
        true
    }
    fn travel(&mut self, to: usize, mode: TravelMode) {
        let cost = self.move_cost(mode);
        self.spend(cost);
        if matches!(mode, TravelMode::Walk | TravelMode::Careful) {
            let threshold = if self.in_combat() {
                8
            } else if self.world == World::Echo {
                7
            } else {
                5
            };
            let bonus = if mode == TravelMode::Careful { 2 } else { 0 }
                + if self.equip == Some(Item::RefinedSoap) {
                    1
                } else {
                    0
                };
            if !self.test("沿路移动", Attribute::Agility, threshold, bonus)
                && self.flag("encountered")
            {
                self.spend(1);
                self.note("移动检定未通过：仍然抵达目标，多消耗 1 体力。");
            }
        }
        if mode == TravelMode::Fly {
            self.take(Item::Fuel, 1);
            self.mark("flown");
        }
        if mode == TravelMode::Elevator {
            let roll = 1 + self.random(6) as i32;
            self.last.rolls.push(Roll {
                label: "电梯异常".into(),
                sides: 6,
                value: roll,
                bonus: 0,
                target: 2,
                defense_sides: 0,
                defense: 0,
                success: roll > 1,
            });
            if roll == 1 {
                self.hp -= 12;
                self.note("门打开时，落脚处出现黑洞。生命 -12；已到达所选楼层。");
            }
        }
        let old_zone = self.echo.zone(self.current);
        self.previous = self.current;
        if self.equip == Some(Item::Boots) {
            self.pits.insert(self.current, self.turn + 3);
            self.note("石靴在出发点留下深坑，可阻挡追踪两回合。");
        }
        self.current = to;
        if self.world == World::Echo && old_zone != self.echo.zone(to) {
            self.reconfigure_echo(Some(old_zone), "跨过区段门后，身后走廊改变");
        }
        if self.at("reception") {
            self.unlock("仓皇离去");
        }
        if NODES[to].roof() {
            self.unlock("擅闯结界");
        }
        if NODES[to].safe {
            for e in &mut self.enemies {
                e.chasing = false;
            }
        }
        if self.at("dungeon") {
            self.unlock("勇探地牢");
        }
        if self.at("geo_office") {
            self.unlock("误陷怪屋");
            self.unlock("拨云见…夜？");
        }
        let p = self.position(to);
        self.note(format!(
            "抵达{}，坐标 ({},{},{})，{}。",
            NODES[to].name,
            p[0],
            p[1],
            p[2],
            self.world.name()
        ));
    }
    fn use_item(&mut self, i: Item) {
        match i {
            Item::Water => {
                self.take(i, 1);
                self.hp = (self.hp + 8).min(self.character.max_hp());
                self.give(Item::EmptyBottle, 1);
                self.note("饮下矿泉水，生命恢复最多 8；空瓶已放回背包。");
            }
            Item::Food => {
                self.take(i, 1);
                self.stamina = (self.stamina + 4).min(self.max_stamina());
                self.note("饼干消耗 1，体力恢复最多 4。");
            }
            Item::Milk => {
                self.take(i, 1);
                self.flags.remove("saponified");
                self.counts.remove("cold_until");
                self.note("牛奶消耗 1 盒，皂化与寒流状态已解除。");
            }
            Item::Leaf => {
                self.equip = Some(i);
                self.note("已把槐叶握在手中。只有继续持握，它才会替你抵消一次致死伤害。");
            }
            Item::Hook => {
                if self.at("dungeon") && !self.has(Item::Stroke) {
                    self.spend(1);
                    if self.test("打捞地牢器物", Attribute::Agility, 7, 2) {
                        self.give(Item::Stroke, 1);
                        self.gain_once("dungeon_fishing", 3);
                        self.unlock("获得一横");
                        self.note("一横圣器已打捞，钓竿保留。");
                    } else {
                        self.note("线没有挂稳，器物仍在水底；钓竿与其他物品没有损失。");
                    }
                } else if let Some(key) = self.rescue_key() {
                    self.spend(1);
                    if self.test("解救学长", Attribute::Agility, 7, 2) {
                        self.mark(&format!("saved_{key}"));
                        self.give(Item::Tear, 1);
                        self.unlock("引渡人");
                        self.gain_once(key, 3);
                        self.note("学长已被救出，留下 1 滴生魂之泪；这位学长不会重复出现。");
                    } else {
                        self.note("钓线错过了落点；学长仍在原处，可调整后再试。");
                    }
                }
            }
            Item::RoughSoap => {
                self.take(i, 1);
                let dest = self
                    .neighbors(self.current)
                    .into_iter()
                    .find(|&n| n == self.previous)
                    .or_else(|| self.neighbors(self.current).first().copied())
                    .unwrap_or(self.current);
                for e in &mut self.enemies {
                    if e.kind == EnemyKind::Wxh && e.node == self.current && e.world == self.world {
                        e.node = dest;
                        e.stun = 4;
                        e.chasing = false;
                    }
                }
                self.mark("soap_story");
                self.unlock("调虎离山");
                self.unlock("皂化池畔");
                self.note("WXH 暂停巡视三回合。皂化池的内间已开放，入口在化学办公间的里世界。");
            }
            Item::Coal => {
                self.take(i, 1);
                let count = self.increment("furnaces", 1);
                self.note(format!(
                    "已点燃高炉 {}/3。矿料是否已投入：{}。",
                    count.min(3),
                    if self.flag("ore_loaded") {
                        "是"
                    } else {
                        "否"
                    }
                ));
                self.finish_furnace();
            }
            Item::Ore => {
                self.take(i, 1);
                self.mark("ore_loaded");
                self.note("矿料已放入高炉，等待三座炉火一并点亮。");
                self.finish_furnace();
            }
            Item::Key => {
                self.mark("hidden_cabinet");
                self.document("thunder3");
                self.document("forged");
                self.unlock("重要说明");
                self.note("隐藏柜已打开，钥匙保留。两页笔记已收录；其中一页的笔迹与你先前见过的不完全相同。");
            }
            Item::EmptyBottle => {
                self.take(i, 1);
                self.give(Item::Alcohol, 1);
                self.note("空瓶交给领料口，取得 1 份醇类物质。");
            }
            Item::Fusion => {
                self.take(i, 1);
                if let Some(j) = self
                    .active_enemies()
                    .into_iter()
                    .find(|&j| self.enemies[j].kind == EnemyKind::Copy)
                {
                    self.enemies[j].hp = 0;
                }
                self.note("相邻的两个复制体发生融合，其中一个实体消失，另一个仍在场。");
            }
            Item::Meter => {
                self.take(i, 1);
                self.hp -= 6;
                self.counts.insert("charge".into(), 1);
                self.unlock("蓄势待发");
                if self.take(Item::Spear, 1) {
                    self.give(Item::ThunderSpear, 1);
                    self.note("ZYY 将铁质花枪改造为钧霆雷枪；它的用途是驱离，不是普通雷电伤害。");
                }
                for e in &mut self.enemies {
                    if e.kind == EnemyKind::Zyy {
                        e.stun = 3;
                    }
                }
                self.note("承受 6 点雷击伤害，雷跃充能至 1；电流计依原案在充能完成后消失。");
            }
            Item::Record => {
                self.mark("used_record");
                self.unlock("你的手记");
                self.note("你的手记已经启用，这项选择不可撤回。探索获得 +1，但实体会更容易找到你；出校门时判定结局。");
            }
            Item::Void => {
                self.mark("used_void");
                for p in [Item::Slug, Item::Spear, Item::RoughSoap, Item::Key] {
                    self.inv.remove(&p);
                    if self.equip == Some(p) {
                        self.equip = None;
                    }
                }
                self.hp = self.character.max_hp();
                self.note(
                    "虚无的造物已经启用，生命恢复；黏痰、花枪、粗制肥皂和深渊之钥从背包中消失。",
                );
            }
            Item::DarkToken => {
                self.mark("dark_active");
                self.note("深渊信物已启用。实体停攻，每回合理智损失 2；合成记录已改变结局条件。");
            }
            _ => {}
        }
    }
    fn finish_furnace(&mut self) {
        if self.counter("furnaces") >= 3 && self.flag("ore_loaded") {
            self.counts.insert("furnaces".into(), 0);
            self.flags.remove("ore_loaded");
            self.give(Item::Boots, 1);
            self.give(Item::Key, 2);
            self.gain_once("furnace_complete", 3);
            self.unlock("炉火如昼");
            self.unlock("涅金之靴");
            self.unlock("幽光秘钥");
            self.note(
                "三座高炉共同完成炼化，取得掘炭石靴与两把深渊之钥。需要时可再次补领矿料与煤炭。",
            );
        }
    }
    fn attack(&mut self, mode: AttackMode) {
        let Some(j) = self.combat_target(mode) else {
            return;
        };
        if mode == AttackMode::Domain {
            let close = self.neighbors(self.current);
            for e in &mut self.enemies {
                if e.hp > 0 && e.world == self.world && close.contains(&e.node) {
                    e.node = self.current;
                }
            }
            self.note("黑域将相邻实体吸至当前地点，随后对目标结算攻击。");
        }
        if mode == AttackMode::Thunder {
            self.spend(1);
            self.counts.insert("charge".into(), 0);
            self.increment("thunder_favors", 1);
            self.unlock("天雷庇佑");
            let dest = self
                .neighbors(self.current)
                .first()
                .copied()
                .unwrap_or(self.current);
            for e in &mut self.enemies {
                if e.node == self.current && e.world == self.world {
                    e.node = dest;
                    e.stun = 3;
                    e.chasing = false;
                }
            }
            self.note("钧霆雷枪落地，周围实体被驱离一格，暂停两回合。雷跃充能已耗尽，庇佑次数增加 1。枪随后收回。" );
            return;
        }
        self.spend(
            1 + if mode == AttackMode::Strike
                && matches!(self.equip, Some(Item::Spear | Item::ThunderSpear))
            {
                1
            } else {
                0
            },
        );
        let attack = match mode {
            AttackMode::Stroke => {
                self.take(Item::Slug, 1);
                10
            }
            AttackMode::Thunder => {
                self.counts.insert("charge".into(), 0);
                self.increment("thunder_favors", 1);
                self.unlock("天雷庇佑");
                self.mark("lightning_guard");
                16
            }
            AttackMode::Assimilate => 6 + self.character.stat(Attribute::Body),
            AttackMode::Domain => 6 + self.character.stat(Attribute::Will),
            _ => {
                4 + self.character.stat(Attribute::Body)
                    + match self.equip {
                        Some(Item::Sword) => 3,
                        Some(Item::Spear | Item::ThunderSpear) => 5,
                        _ => 0,
                    }
            }
        };
        let attack = attack
            + if self.committed == Some(Tactic::Opening) {
                2
            } else {
                0
            };
        let damage = self.opposed("攻击与防御", attack, 4);
        if self.enemies[j].kind.mortal() {
            self.enemies[j].hp -= damage;
            self.note(format!(
                "{} 生命 -{}。",
                self.enemies[j].kind.name(),
                damage
            ));
            if self.enemies[j].hp <= 0 {
                if self.enemies[j].kind == EnemyKind::Assimilated {
                    self.give(Item::Fat, 2);
                    self.unlock("“防卫过当”");
                }
                self.gain_once(&format!("defeat_{j}"), 2);
                self.note("实体已失去行动能力，当前回合不会再攻击。");
            }
        } else {
            self.note("这位实体不能在本章通过普通生命伤害击败；利用器物效果争取离开的机会。");
        }
        if matches!(mode, AttackMode::Lamp | AttackMode::Stroke) {
            let dest = if self.neighbors(self.current).contains(&self.previous) {
                self.previous
            } else {
                self.neighbors(self.current)
                    .first()
                    .copied()
                    .unwrap_or(self.current)
            };
            self.enemies[j].node = dest;
            self.enemies[j].stun = 3;
            self.enemies[j].chasing = false;
            self.note(format!(
                "目标被击退至{}，接下来两回合停止行动。",
                NODES[dest].name
            ));
        }
        if mode == AttackMode::Lamp {
            let value = 1 + self.random(2) as i32;
            self.last.rolls.push(Roll {
                label: "酒精灯·造化".into(),
                sides: 2,
                value,
                bonus: 0,
                target: 2,
                defense_sides: 0,
                defense: 0,
                success: value == 2,
            });
            if value == 2 {
                self.flags.remove("saponified");
                self.unlock("巧同造化");
            }
        }
    }
    fn tick(&mut self, guard: bool) {
        self.turn += 1;
        if self.in_combat() {
            self.mark("encountered");
        }
        if self.world == World::Echo && !NODES[self.current].safe {
            self.sanity -= 1;
            self.note("里世界持续侵蚀：理智 -1。");
        }
        if self.world == World::Echo {
            let count = self.increment("echo_actions", 1);
            if count % 24 == 0 {
                self.reconfigure_echo(Some(self.echo.zone(self.current)), "钟声过后，本区重新排列");
            }
        }
        if self.flag("dark_active") {
            self.sanity -= 2;
            self.note("深渊信物持续侵蚀：理智 -2。");
        }
        if self.flag("used_void") && self.hp < self.character.max_hp() {
            self.hp += 1;
            self.note("虚无的造物持续恢复：生命 +1。");
        }
        if self.flag("used_record") && self.turn % 6 == 0 {
            self.spawn(EnemyKind::Copy);
            self.note("手记上的字迹又深了一层，复制体循着记录来到你身旁。");
        }
        if self.flag("saponified") && self.stamina > 0 {
            self.stamina -= 1;
            self.note("皂化持续影响：体力额外 -1；牛奶可解除。");
        }
        for j in 0..self.enemies.len() {
            if self.enemies[j].hp <= 0 || self.enemies[j].world != self.world {
                continue;
            }
            if self.enemies[j].stun > 0 {
                self.enemies[j].stun -= 1;
                continue;
            }
            if self.flag("dark_active") {
                continue;
            }
            if self.enemies[j].node != self.current {
                if self.enemies[j].chasing && self.turn % 2 == 0 && !NODES[self.current].safe {
                    let start = self.enemies[j].node;
                    if self
                        .pits
                        .get(&start)
                        .is_some_and(|&until| until > self.turn)
                    {
                        continue;
                    }
                    if let Some(step) = self.shortest_step(start, self.current) {
                        self.enemies[j].node = step;
                        self.note(format!(
                            "{} 正沿走廊追来；下次同处一地时会攻击。",
                            self.enemies[j].kind.name()
                        ));
                    }
                }
                continue;
            }
            let kind = self.enemies[j].kind;
            if kind == EnemyKind::Zyy && self.flag("lightning_guard") {
                continue;
            }
            let defense = if kind.mental() {
                2 + self.character.stat(Attribute::Will)
            } else {
                2 + self.character.stat(Attribute::Agility)
                    + if self.equip == Some(Item::Armor) {
                        3
                    } else {
                        0
                    }
            } + if guard { 4 } else { 0 }
                + if self.easy { 2 } else { 0 };
            let defense = defense
                + (if self.committed == Some(Tactic::Guard) {
                    3
                } else {
                    0
                })
                + (if kind.mental() && self.committed == Some(Tactic::Composure) {
                    2
                } else {
                    0
                });
            let mut dmg =
                self.opposed(&format!("{} 的下一击", kind.name()), kind.attack(), defense);
            if kind == EnemyKind::Ll && self.equip == Some(Item::Wenjin) {
                dmg = (dmg + 3) / 4;
            }
            if kind.mental() {
                self.sanity -= dmg;
            } else {
                self.hp -= dmg;
            }
            self.note(format!(
                "{} 结算：{} -{}。",
                kind.name(),
                if kind.mental() { "理智" } else { "生命" },
                dmg
            ));
            if kind == EnemyKind::Wxh && dmg > 0 {
                self.mark("saponified");
            }
            if kind == EnemyKind::Ll && self.equip != Some(Item::Wenjin) {
                self.counts.insert("cold_until".into(), self.turn + 3);
                self.note("寒流使接下来的普通移动多耗 1 体力；持握问津可以免疫。");
            }
            if self.hp <= 0 || self.sanity <= 0 {
                if matches!(kind, EnemyKind::Copy | EnemyKind::Assimilated) {
                    self.increment("copy_deaths", 1);
                    self.unlock(if kind == EnemyKind::Copy {
                        "“恐梦症”"
                    } else {
                        "趋同进化"
                    });
                }
                if kind == EnemyKind::Wxh {
                    self.unlock("刳脂剔膏");
                }
                self.rebirth(
                    &format!("{} 的攻击使你失去意识", kind.name()),
                    kind == EnemyKind::Wxh && self.flag("soap_story"),
                );
                break;
            }
        }
        self.flags.remove("lightning_guard");
        self.pits.retain(|_, until| *until > self.turn);
    }
}
