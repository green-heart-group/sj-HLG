use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum World {
    Surface,
    Echo,
}
impl World {
    pub fn index(self) -> usize {
        if self == Self::Surface {
            0
        } else {
            1
        }
    }
    pub fn mask(self) -> u8 {
        1 << self.index()
    }
    pub fn other(self) -> Self {
        if self == Self::Surface {
            Self::Echo
        } else {
            Self::Surface
        }
    }
    pub fn name(self) -> &'static str {
        if self == Self::Surface {
            "表世界"
        } else {
            "里世界"
        }
    }
}
include!("item_enum.rs");
#[derive(Debug, Deserialize)]
pub struct ItemInfo {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub icon: usize,
    pub source: String,
    pub text: String,
    pub effect: String,
    pub implementation_authority: Vec<String>,
}
pub static ITEMS: LazyLock<Vec<ItemInfo>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/items.json")).expect("item catalog")
});
pub const RELICS: [Item; 8] = [
    Item::Wenjin,
    Item::Soap,
    Item::Thunder,
    Item::Diploid,
    Item::Stroke,
    Item::Boots,
    Item::Wheel,
    Item::Domain,
];
impl Item {
    pub fn info(self) -> &'static ItemInfo {
        &ITEMS[self as usize]
    }
    pub fn name(self) -> &'static str {
        &self.info().name
    }
    pub fn description(self) -> &'static str {
        &self.info().text
    }
    pub fn effect(self) -> &'static str {
        &self.info().effect
    }
    pub fn is_relic(self) -> bool {
        RELICS.contains(&self)
    }
    pub fn equippable(self) -> bool {
        self.is_relic()
            || matches!(
                self,
                Self::Lamp
                    | Self::Sword
                    | Self::Armor
                    | Self::Spear
                    | Self::ThunderSpear
                    | Self::RefinedSoap
                    | Self::Leaf
            )
    }
    pub fn retained(self) -> bool {
        self.is_relic() || self.info().kind == "结局"
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub raw_coordinate: String,
    pub source_line: usize,
    pub source: String,
    pub correction: Option<String>,
    pub provisional: bool,
    pub interior: bool,
    pub parent: Option<String>,
    pub worlds: u8,
    pub safe: bool,
    pub mirror: bool,
}
impl Node {
    pub fn exists(&self, w: World) -> bool {
        self.worlds & w.mask() != 0
    }
    pub fn title(&self, _w: World) -> &str {
        &self.name
    }
    pub fn scene(&self) -> usize {
        match self.id.as_str() {
            "clock" => 6,
            "lockers" => 7,
            "clinic" => 11,
            "canteen" => 12,
            "classroom" => 17,
            "company" => 8,
            "principal" => 24,
            "training" | "altar" => 23,
            "gate" | "fountain" | "garden" => 0,
            "storage" | "chem_lab" | "chem_inner" | "saponify" | "physics_lab" | "electric_lab"
            | "bio_lab" | "specimen" | "captive" => 10,
            "geo_office" | "dungeon" => 22,
            _ if self.name == "学生公司" => 8,
            _ if self.name == "喷泉" => 0,
            _ if self.name == "综合楼顶层" => 23,
            _ if self.name.contains("走廊") || self.name.contains("电梯") => 5,
            _ => 3,
        }
    }
    pub fn roof(&self) -> bool {
        self.name == "综合楼顶层"
    }
    pub fn outside(&self) -> bool {
        self.z == 1 && matches!(self.id.as_str(), "gate" | "fountain" | "garden" | "jinfan")
    }
    pub fn description(&self, w: World) -> String {
        crate::story::scene(self, w)
    }
}
#[derive(Debug, Deserialize)]
pub struct Edge {
    pub a: usize,
    pub b: usize,
    pub worlds: u8,
    pub source: String,
    pub rationale: String,
}
#[derive(Deserialize)]
struct MapData {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}
static MAP: LazyLock<MapData> =
    LazyLock::new(|| serde_json::from_str(include_str!("../data/map.json")).expect("map data"));
pub static SOURCE_NODE_COUNT: LazyLock<usize> = LazyLock::new(|| MAP.nodes.len());
pub static NODES: LazyLock<Vec<Node>> = LazyLock::new(|| {
    let mut nodes = MAP.nodes.clone();
    for z in 0..crate::echo::ZONES {
        for i in 0..crate::echo::CELLS {
            let name = match i {
                0 => "学生公司",
                1 => "喷泉",
                63 => "综合楼顶层",
                _ => ["复制长廊", "南走廊", "北走廊", "实验楼廊道"][i % 4],
            };
            nodes.push(Node {
                id: format!("echo_{z:02}_{i:02}"),
                name: name.into(),
                x: 0,
                y: 0,
                z: 0,
                raw_coordinate: "动态里世界坐标".into(),
                source_line: 0,
                source: "U13:复制校园房间与区段".into(),
                correction: None,
                provisional: false,
                interior: false,
                parent: None,
                worlds: 2,
                safe: i == 0,
                mirror: i == 1,
            });
        }
    }
    nodes
});
pub static EDGES: LazyLock<&'static [Edge]> = LazyLock::new(|| &MAP.edges);
pub fn node(key: &str) -> usize {
    NODES
        .iter()
        .position(|n| n.id == key)
        .unwrap_or_else(|| panic!("unknown node {key}"))
}
pub fn neighbors(id: usize, w: World) -> Vec<usize> {
    EDGES
        .iter()
        .filter(|e| e.worlds & w.mask() != 0)
        .filter_map(|e| {
            if e.a == id {
                Some(e.b)
            } else if e.b == id {
                Some(e.a)
            } else {
                None
            }
        })
        .filter(|&n| NODES[n].exists(w))
        .collect()
}
#[derive(Debug, Deserialize)]
pub struct Achievement {
    pub id: String,
    pub kind: String,
    pub name: String,
    pub root: bool,
    pub revision_marker: String,
    pub text: String,
    pub condition: String,
    pub parent: String,
    pub placeholder: bool,
    pub source_start: usize,
    pub source_end: usize,
    pub text_sha256: String,
}
pub static ACHIEVEMENTS: LazyLock<Vec<Achievement>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/achievements.json")).expect("achievement source")
});
#[derive(Debug, Deserialize)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub text: String,
    pub source: String,
}
pub static DOCUMENTS: LazyLock<Vec<Document>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../data/documents.json")).expect("documents")
});
