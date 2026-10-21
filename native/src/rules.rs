use crate::{cards::Tactic, character::*, content::*};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
pub const SAVE_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ending {
    Escape,
    Publish,
    Annihilation,
    Withered,
    Loop,
    Mercy,
}
impl Ending {
    pub fn name(self) -> &'static str {
        match self {
            Self::Escape => "逃离",
            Self::Publish => "公之于众",
            Self::Annihilation => "堙灭",
            Self::Withered => "干枯",
            Self::Loop => "循环",
            Self::Mercy => "仁主",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
        Self::Escape=>"你似乎了解了事情的全貌，但是，无知是幸福的，不是么？",
        Self::Publish=>"你终于也成为了一位活在笔记中的学长；等待下一位无知的学生，把你从笔记中解脱出来……",
        Self::Annihilation=>"凝视深渊的你，也必定被深渊凝视；当你借助深渊的力量，也必定被深渊拖入无底地狱。",
        Self::Withered=>"你不应当依赖于未知的事物，因为即使是 ZYY 那看似正义的举动，对你来说也是不可承受之重。别忘了他的笔记里说过什么，凡人。",
        Self::Loop=>"你知晓了这里的秘密，但并没有选择忘记；深渊不愿接受你尚存的理智，那么，你只能再次回到循环的始端，……直到你疯狂为止。",
        Self::Mercy=>"在探寻真相的道路上，你顺手拯救了七条原本永世不得超脱的魂魄。生魂淌下的泪水铭记你的恩德，庇佑你免于诸仙困囿的恶咒；七盏魂灯，将指引仁慈的你逃离这永恒的炼狱。",
    }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Playing,
    Ended(Ending),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyKind {
    Wxh,
    Zyy,
    Copy,
    Assimilated,
    Ll,
    Sgf,
    Hjf,
    Ljx,
}
impl EnemyKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Wxh => "WXH · 年级主任",
            Self::Zyy => "ZYY · 雷法之主",
            Self::Copy => "复制体",
            Self::Assimilated => "被同化者",
            Self::Ll => "LL · 语文老师",
            Self::Sgf => "SGF · 地理老师",
            Self::Hjf => "HJF · 飞轮之主",
            Self::Ljx => "LJX · 历史老师",
        }
    }
    pub fn mortal(self) -> bool {
        matches!(self, Self::Copy | Self::Assimilated)
    }
    pub fn mental(self) -> bool {
        matches!(self, Self::Ll | Self::Hjf)
    }
    pub fn attack(self) -> i32 {
        match self {
            Self::Wxh => 10,
            Self::Zyy => 12,
            Self::Copy => 7,
            Self::Assimilated => 6,
            Self::Ll => 14,
            Self::Sgf => 9,
            Self::Hjf => 14,
            Self::Ljx => 11,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub node: usize,
    pub world: World,
    pub hp: i32,
    pub stun: u32,
    pub chasing: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Roll {
    pub label: String,
    pub sides: i32,
    pub value: i32,
    pub bonus: i32,
    pub target: i32,
    pub defense_sides: i32,
    pub defense: i32,
    pub success: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Resolution {
    pub title: String,
    pub story: String,
    pub rolls: Vec<Roll>,
    pub notes: Vec<String>,
    #[serde(default)]
    pub changes: Vec<String>,
    pub accepted: bool,
    pub major: bool,
    pub motion: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub version: u32,
    pub seed: u64,
    pub rng: u64,
    pub easy: bool,
    pub character: Character,
    pub current: usize,
    pub world: World,
    pub hp: i32,
    pub stamina: i32,
    pub sanity: i32,
    pub turn: u32,
    pub inv: BTreeMap<Item, u16>,
    pub equip: Option<Item>,
    pub known: [BTreeSet<usize>; 2],
    pub visited: [BTreeSet<usize>; 2],
    pub flags: BTreeSet<String>,
    pub counts: BTreeMap<String, u32>,
    pub clues: BTreeSet<String>,
    pub achievements: BTreeSet<String>,
    pub enemies: Vec<Enemy>,
    pub pits: BTreeMap<usize, u32>,
    pub log: VecDeque<String>,
    pub status: Status,
    pub mail: Option<BTreeMap<Item, u16>>,
    pub last: Resolution,
    pub previous: usize,
    pub hand: Vec<Tactic>,
    pub draw_pile: Vec<Tactic>,
    pub discard_pile: Vec<Tactic>,
    pub committed: Option<Tactic>,
    pub echo: crate::echo::EchoLayout,
    pub entry_surface: usize,
}
impl Game {
    pub fn new(seed: u64, easy: bool) -> Self {
        Self::with_character(seed, easy, Character::default())
    }
    pub fn with_character(seed: u64, easy: bool, character: Character) -> Self {
        let mut g = Self {
            version: SAVE_VERSION,
            seed,
            rng: seed.wrapping_add(0x9e3779b97f4a7c15).max(1),
            easy,
            hp: character.max_hp(),
            stamina: character.max_stamina(),
            character,
            current: node("gate"),
            previous: node("gate"),
            world: World::Surface,
            sanity: 100,
            turn: 0,
            inv: BTreeMap::new(),
            equip: None,
            known: Default::default(),
            visited: Default::default(),
            flags: BTreeSet::new(),
            counts: BTreeMap::new(),
            clues: BTreeSet::new(),
            achievements: BTreeSet::new(),
            enemies: vec![],
            pits: BTreeMap::new(),
            log: VecDeque::new(),
            status: Status::Playing,
            mail: None,
            last: Resolution::default(),
            hand: vec![],
            draw_pile: vec![
                Tactic::Observe,
                Tactic::Quiet,
                Tactic::Composure,
                Tactic::Opening,
                Tactic::Guard,
                Tactic::Observe,
                Tactic::Quiet,
                Tactic::Composure,
                Tactic::Opening,
                Tactic::Guard,
            ],
            discard_pile: vec![],
            committed: None,
            echo: crate::echo::EchoLayout::new(seed),
            entry_surface: node("fountain"),
        };
        g.refill_hand();
        g.give(Item::Food, 2);
        g.give(Item::Water, 1);
        g.unlock("主线成就");
        g.unlock("踏入校园");
        g.reveal();
        g.last=Resolution {title:"铁门在身后合上".into(),story:"值班室仍亮着灯，窗口后面放着一本登记簿。先核对自己的去向，再决定如何走过这座白天十分熟悉的校园。".into(),notes:vec!["先到传达室查看离校须知。选择移动卡后可预览消耗，再确认掷骰。".into()],accepted:true,major:true,motion:"arrival".into(),..Default::default()};
        g
    }
    pub fn playing(&self) -> bool {
        self.status == Status::Playing
    }
    pub fn flag(&self, k: &str) -> bool {
        self.flags.contains(k)
    }
    pub fn mark(&mut self, k: &str) {
        self.flags.insert(k.into());
    }
    pub fn count(&self, i: Item) -> u16 {
        *self.inv.get(&i).unwrap_or(&0)
    }
    pub fn has(&self, i: Item) -> bool {
        self.count(i) > 0
    }
    pub fn counter(&self, k: &str) -> u32 {
        *self.counts.get(k).unwrap_or(&0)
    }
    pub fn increment(&mut self, k: &str, n: u32) -> u32 {
        let v = self.counts.entry(k.into()).or_default();
        *v += n;
        *v
    }
    pub fn give(&mut self, i: Item, n: u16) {
        let first = !self.has(i);
        *self.inv.entry(i).or_default() += n;
        if first && (i.equippable() || matches!(i.info().kind.as_str(), "器物" | "权限" | "结局"))
        {
            self.last.major = true;
        }
        if first && i.is_relic() {
            self.mark(&format!("found_{i:?}"));
            self.gain_once(&format!("relic_{i:?}"), 3);
        }
    }
    pub(crate) fn take(&mut self, i: Item, n: u16) -> bool {
        if self.count(i) < n {
            return false;
        }
        let c = self.inv.get_mut(&i).unwrap();
        *c -= n;
        if *c == 0 {
            self.inv.remove(&i);
            if self.equip == Some(i) {
                self.equip = None;
            }
        }
        true
    }
    pub fn max_stamina(&self) -> i32 {
        self.character.max_stamina()
            + if self.equip == Some(Item::Diploid) {
                3
            } else {
                0
            }
    }
    pub fn relic_count(&self) -> usize {
        RELICS.iter().filter(|&&i| self.has(i)).count()
    }
    pub fn active_enemies(&self) -> Vec<usize> {
        self.enemies
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                e.hp > 0 && e.node == self.current && e.world == self.world && e.stun == 0
            })
            .map(|(i, _)| i)
            .collect()
    }
    pub fn in_combat(&self) -> bool {
        !self.active_enemies().is_empty()
    }
    pub fn combat_target(&self, mode: crate::cards::AttackMode) -> Option<usize> {
        use crate::cards::AttackMode;
        if let Some(j) = self.active_enemies().first() {
            return Some(*j);
        }
        self.enemies
            .iter()
            .enumerate()
            .find(|(_, e)| {
                if e.hp <= 0 || e.world != self.world {
                    return false;
                }
                match mode {
                    AttackMode::Domain => self.neighbors(self.current).contains(&e.node),
                    AttackMode::Stroke => {
                        let a = self.position(self.current);
                        let b = self.position(e.node);
                        a[2] == b[2]
                            && (a[0] == b[0] || a[1] == b[1])
                            && self.known[self.world.index()].contains(&e.node)
                    }
                    _ => false,
                }
            })
            .map(|(i, _)| i)
    }
    pub fn at(&self, k: &str) -> bool {
        NODES[self.current].id == k
    }
    pub fn note(&mut self, s: impl Into<String>) {
        let s = s.into();
        self.last.notes.push(s.clone());
        self.log.push_back(s);
        while self.log.len() > 100 {
            self.log.pop_front();
        }
    }
    pub(crate) fn unlock(&mut self, name: &str) {
        if self.achievements.insert(name.into())
            && !matches!(name, "主线成就" | "踏入校园" | "彩蛋成就")
        {
            self.note(format!("成就已点亮：{name}"));
        }
    }
    pub(crate) fn document(&mut self, id: &str) {
        if self.clues.insert(id.into()) {
            self.gain_once(&format!("read_{id}"), 2);
            if let Some(d) = DOCUMENTS.iter().find(|d| d.id == id) {
                self.note(format!("手记已收录：{}。可在档案中逐页阅读。", d.title));
            }
        }
    }
    pub(crate) fn gain_once(&mut self, key: &str, xp: u32) {
        if self.flags.insert(format!("xp_{key}")) {
            self.character.gain(xp);
        }
    }
    pub(crate) fn random(&mut self, n: u64) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x % n
    }
    pub(crate) fn test(
        &mut self,
        label: &str,
        attr: Attribute,
        target: i32,
        modifier: i32,
    ) -> bool {
        let value = 1 + self.random(6) as i32;
        let prepared = if [
            "打捞地牢器物",
            "解救学长",
            "辨认并击除复制体",
            "破除祭坛",
            "黑域中的触碰",
            "核对记录",
        ]
        .contains(&label)
            && self.flags.remove("prepared")
        {
            2
        } else {
            0
        };
        let bonus = self.character.stat(attr)
            + modifier
            + prepared
            + (if self.easy { 1 } else { 0 })
            + (if attr == Attribute::Insight && self.flag("used_record") {
                1
            } else {
                0
            })
            + self.committed.map_or(0, |t| t.attribute_bonus(attr));
        let success = value + bonus >= target;
        self.last.rolls.push(Roll {
            label: label.into(),
            sides: 6,
            value,
            bonus,
            target,
            defense_sides: 0,
            defense: 0,
            success,
        });
        success
    }
    pub(crate) fn opposed(&mut self, label: &str, attack: i32, defense: i32) -> i32 {
        let a = 1 + self.random(attack.max(1) as u64) as i32;
        let d = 1 + self.random(defense.max(1) as u64) as i32;
        self.last.rolls.push(Roll {
            label: label.into(),
            sides: attack,
            value: a,
            bonus: 0,
            target: d,
            defense_sides: defense,
            defense: d,
            success: a > d,
        });
        (a - d).max(0)
    }
    pub(crate) fn spend(&mut self, n: i32) {
        let n = n.max(0);
        let lack = (n - self.stamina).max(0);
        self.stamina = (self.stamina - n).max(0);
        if lack > 0 {
            self.hp -= lack * 2;
            self.note(format!("体力不足，透支 {} 点生命。", lack * 2));
        }
    }
    pub(crate) fn spawn(&mut self, kind: EnemyKind) {
        if !self
            .enemies
            .iter()
            .any(|e| e.kind == kind && e.node == self.current && e.world == self.world && e.hp > 0)
        {
            self.enemies.push(Enemy {
                kind,
                node: self.current,
                world: self.world,
                hp: if kind.mortal() { 12 } else { 999 },
                stun: 1,
                chasing: kind == EnemyKind::Wxh,
            });
            self.mark("encountered");
            self.last.major = true;
            self.note(format!(
                "遭遇 {}。它的下一步意图已显示，选择应对卡后才推进回合。",
                kind.name()
            ));
        }
    }
    pub fn reveal(&mut self) {
        let wi = self.world.index();
        if self.visited[wi].insert(self.current) {
            self.gain_once(&format!("visit_{wi}_{}", self.current), 1);
        }
        self.known[wi].insert(self.current);
        let adjacent = self.neighbors(self.current);
        self.known[wi].extend(adjacent.clone());
        if self.equip == Some(Item::Soap) {
            for n in adjacent {
                let more = self.neighbors(n);
                self.known[wi].extend(more);
            }
        }
        if self.equip == Some(Item::Stroke) {
            let c = self.position(self.current);
            for (i, n) in NODES.iter().enumerate() {
                let p = self.position(i);
                if p[2] == c[2] && (p[0] == c[0] || p[1] == c[1]) && n.exists(self.world) {
                    self.known[wi].insert(i);
                }
            }
        }
        if self.equip == Some(Item::Thunder) {
            let c = self.position(self.current);
            for (i, n) in NODES.iter().enumerate() {
                let p = self.position(i);
                if p[0] == c[0] && p[1] == c[1] && p[2] > c[2] && n.exists(self.world) {
                    self.known[wi].insert(i);
                }
            }
        }
        if self.equip == Some(Item::Wheel)
            && (NODES[self.current].roof() || NODES[self.current].outside())
        {
            for (i, n) in NODES.iter().enumerate() {
                if n.roof() && n.exists(self.world) {
                    self.known[wi].insert(i);
                }
            }
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != SAVE_VERSION {
            return Err("此存档属于其他规则版本；旧存档保留在原目录。".into());
        }
        self.echo.validate()?;
        if self.entry_surface >= *SOURCE_NODE_COUNT
            || !NODES[self.entry_surface].exists(World::Surface)
        {
            return Err("镜面出口异常".into());
        }
        if self.current >= NODES.len()
            || !NODES[self.current].exists(self.world)
            || self.previous >= NODES.len()
        {
            return Err("存档位置异常".into());
        }
        if !(0..=100).contains(&self.sanity)
            || self.hp < 1
            || self.hp > self.character.max_hp()
            || self.stamina < 0
            || self.stamina > self.max_stamina()
        {
            return Err("存档属性异常".into());
        }
        if self.character.values.iter().any(|&v| !(1..=8).contains(&v)) {
            return Err("角色属性异常".into());
        }
        if self.equip.is_some_and(|i| !self.has(i)) {
            return Err("持握物品不在背包".into());
        }
        if self.hand.len() > 3
            || self.hand.len() + self.draw_pile.len() + self.discard_pile.len() != 10
        {
            return Err("手牌或牌库数量异常".into());
        }
        if self
            .known
            .iter()
            .chain(self.visited.iter())
            .any(|s| s.iter().any(|&i| i >= NODES.len()))
            || self.enemies.iter().any(|e| e.node >= NODES.len())
        {
            return Err("存档地图索引异常".into());
        }
        Ok(())
    }
    pub(crate) fn refill_hand(&mut self) {
        while self.hand.len() < 3 {
            if self.draw_pile.is_empty() {
                self.draw_pile = std::mem::take(&mut self.discard_pile);
            }
            if self.draw_pile.is_empty() {
                break;
            }
            let i = self.random(self.draw_pile.len() as u64) as usize;
            self.hand.push(self.draw_pile.remove(i));
        }
    }
    pub fn ending(&self) -> Ending {
        if self.count(Item::Tear) >= 7 {
            Ending::Mercy
        } else if self.flag("made_dark") {
            Ending::Annihilation
        } else if self.counter("thunder_favors") >= 4 {
            Ending::Withered
        } else if self.flag("used_record") {
            Ending::Publish
        } else if self.flag("used_void") {
            Ending::Escape
        } else {
            Ending::Loop
        }
    }
    pub(crate) fn rebirth(&mut self, reason: &str, pool: bool) {
        if self.equip == Some(Item::Leaf) && self.take(Item::Leaf, 1) {
            self.hp = self.character.max_hp() / 2;
            self.sanity = self.sanity.max(20);
            self.increment("leaf_saves", 1);
            self.unlock("命数流转");
            self.note(
                "金色槐叶在掌心里变得透明，替你拦住了这次致死伤害。生命恢复至一半，槐叶消耗 1。",
            );
            return;
        }
        self.increment("deaths", 1);
        self.unlock("循环往复");
        self.last.major = true;
        let old = self.inv.clone();
        self.inv = self.mail.clone().unwrap_or_default();
        for (i, n) in old {
            if i.retained() {
                *self.inv.entry(i).or_default() = n;
            }
        }
        self.equip = self.equip.filter(|&i| self.has(i));
        self.flags.remove("saponified");
        self.counts.remove("cold_until");
        self.hp = self.character.max_hp();
        self.stamina = self.max_stamina();
        self.sanity = 100;
        self.current = if pool {
            node("saponify")
        } else if self.mail.is_some() {
            node("clock")
        } else {
            node("gate")
        };
        self.world = if pool { World::Echo } else { World::Surface };
        self.reconfigure_echo(None, "再次醒来时，里世界的道路已经改变");
        for e in &mut self.enemies {
            e.stun = 3;
            e.chasing = false;
        }
        self.note(format!(
            "{reason}。你再次醒来时，已经回到{}。信物、结局物品和邮筒寄存物保留，剧情与成长继续。",
            NODES[self.current].name
        ));
        if pool {
            let roll = 1 + self.random(7) as i32;
            self.last.rolls.push(Roll {
                label: "皂化成品判定".into(),
                sides: 7,
                value: roll,
                bonus: 0,
                target: 7,
                defense_sides: 0,
                defense: 0,
                success: roll == 7,
            });
            if roll == 7 {
                self.give(Item::Soap, 1);
                self.unlock("化身为皂");
                self.note("整理箱里留下了一块细研香皂。成品检定 d7 = 7，信物已取得。");
            } else {
                self.give(Item::RefinedSoap, 1);
                self.unlock("精制肥皂");
                self.note(
                    "这次留下的是精制肥皂。原案成品概率为 1/7；可继续实验，已获得的信物不会丢失。",
                );
            }
        }
    }
}
