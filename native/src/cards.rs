use crate::{character::Attribute, content::Item};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tactic {
    Observe,
    Quiet,
    Composure,
    Opening,
    Guard,
}
impl Tactic {
    pub fn name(self) -> &'static str {
        match self {
            Self::Observe => "细看一遍",
            Self::Quiet => "放轻脚步",
            Self::Composure => "稳住心神",
            Self::Opening => "抓住空隙",
            Self::Guard => "留出退路",
        }
    }
    pub fn effect(self) -> &'static str {
        match self {
            Self::Observe => "洞察检定 +2",
            Self::Quiet => "机敏检定 +2",
            Self::Composure => "定力检定 +2，精神防御骰 +2 面",
            Self::Opening => "体魄检定 +2，攻击骰 +2 面",
            Self::Guard => "本回合防御骰 +3 面",
        }
    }
    pub fn attribute_bonus(self, a: Attribute) -> i32 {
        if matches!(
            (self, a),
            (Self::Observe, Attribute::Insight)
                | (Self::Quiet, Attribute::Agility)
                | (Self::Composure, Attribute::Will)
                | (Self::Opening, Attribute::Body)
        ) {
            2
        } else {
            0
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Approach {
    Search,
    Study,
    Careful,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TravelMode {
    Walk,
    Careful,
    Slip,
    Leap,
    Fly,
    Elevator,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttackMode {
    Strike,
    Lamp,
    Stroke,
    Assimilate,
    Domain,
    Thunder,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Travel { to: usize, mode: TravelMode },
    Explore(Approach),
    Quest(String),
    Shift,
    Rest,
    SaveMail,
    Use(Item),
    Equip(Option<Item>),
    Craft(Item),
    Attack(AttackMode),
    Defend,
    Barter(Item),
    Train(Attribute),
    Reshape,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardTab {
    Explore,
    Move,
    Combat,
    Items,
}
#[derive(Debug, Clone)]
pub struct Card {
    pub action: Action,
    pub title: String,
    pub story: String,
    pub rule: String,
    pub cost: String,
    pub enabled: bool,
    pub reason: String,
    pub icon: Option<Item>,
    pub source: String,
}
impl Card {
    pub fn new(
        action: Action,
        title: &str,
        story: &str,
        rule: &str,
        cost: &str,
        source: &str,
    ) -> Self {
        Self {
            action,
            title: title.into(),
            story: story.into(),
            rule: rule.into(),
            cost: cost.into(),
            enabled: true,
            reason: String::new(),
            icon: None,
            source: source.into(),
        }
    }
    pub fn require(mut self, ok: bool, reason: impl Into<String>) -> Self {
        if !ok {
            self.enabled = false;
            self.reason = reason.into();
        }
        self
    }
    pub fn item(mut self, item: Item) -> Self {
        self.icon = Some(item);
        self
    }
}
pub const RECIPES: &[(Item, &[(Item, u16)])] = &[
    (
        Item::Lamp,
        &[(Item::Ink, 1), (Item::Solution, 1), (Item::Matches, 1)],
    ),
    (Item::RoughSoap, &[(Item::Fat, 1), (Item::Alkali, 1)]),
    (Item::RefinedSoap, &[(Item::RoughSoap, 1)]),
    (
        Item::Meter,
        &[
            (Item::Gauge, 1),
            (Item::Arrow, 1),
            (Item::Clip, 1),
            (Item::Wire, 1),
        ],
    ),
    (Item::Fusion, &[(Item::Alcohol, 2)]),
    (Item::Record, &[(Item::Notebook, 1), (Item::Spark, 1)]),
    (Item::Void, &[(Item::Notebook, 1), (Item::Blood, 1)]),
    (
        Item::DarkToken,
        &[(Item::Boots, 1), (Item::Key, 2), (Item::Spark, 1)],
    ),
];
pub fn recipe(item: Item) -> Option<&'static [(Item, u16)]> {
    RECIPES.iter().find(|(i, _)| *i == item).map(|(_, v)| *v)
}
