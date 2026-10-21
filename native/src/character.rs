use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(usize)]
pub enum Attribute {
    Body,
    Agility,
    Insight,
    Will,
}
pub const ATTRIBUTES: [Attribute; 4] = [
    Attribute::Body,
    Attribute::Agility,
    Attribute::Insight,
    Attribute::Will,
];
impl Attribute {
    pub fn name(self) -> &'static str {
        match self {
            Self::Body => "体魄",
            Self::Agility => "机敏",
            Self::Insight => "洞察",
            Self::Will => "定力",
        }
    }
    pub fn meaning(self) -> &'static str {
        match self {
            Self::Body => "生命上限与攻击骰；决定正面冲突时能撑多久。",
            Self::Agility => "体力上限、移动检定与防御骰；帮助你在追踪中选择路线。",
            Self::Insight => "探索、阅读和实验检定；决定能否在一次行动中查清关键细节。",
            Self::Will => "精神防御与穿镜检定；帮助你承受神智侵蚀。",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    pub values: [u8; 4],
    pub xp: u32,
    pub growth: u8,
    pub level: u8,
}
impl Character {
    pub fn new(values: [u8; 4]) -> Result<Self, String> {
        if values.iter().any(|&v| !(1..=5).contains(&v))
            || values.iter().map(|&x| x as u32).sum::<u32>() != 12
        {
            return Err("四项属性各为 1–5，合计必须为 12。".into());
        }
        Ok(Self {
            values,
            xp: 0,
            growth: 0,
            level: 1,
        })
    }
    pub fn stat(&self, a: Attribute) -> i32 {
        self.values[a as usize] as i32
    }
    pub fn max_hp(&self) -> i32 {
        20 + self.stat(Attribute::Body) * 5
    }
    pub fn max_stamina(&self) -> i32 {
        8 + self.stat(Attribute::Agility)
    }
    pub fn gain(&mut self, n: u32) {
        self.xp += n;
        while self.xp >= self.level as u32 * 8 && self.level < 9 {
            self.level += 1;
            self.growth += 1;
        }
    }
    pub fn train(&mut self, a: Attribute) -> bool {
        if self.growth == 0 || self.values[a as usize] >= 8 {
            return false;
        }
        self.values[a as usize] += 1;
        self.growth -= 1;
        true
    }
}
impl Default for Character {
    fn default() -> Self {
        Self::new([3, 3, 3, 3]).unwrap()
    }
}
