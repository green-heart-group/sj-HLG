use crate::{
    art::{self, alpha, c},
    App,
};
use hlg_core::{cards::*, character::*, content::*, feedback, game::*};
use macroquad::prelude::*;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Title,
    Create,
    Game,
    Pause,
    Inventory,
    Journal,
    Character,
    Reader,
    End,
}
pub enum Intent {
    None,
    New,
    Begin,
    Continue,
    Screen(Screen),
    Back,
    Floor(i32),
    Select(usize),
    Group(Vec<usize>),
    Tab(CardTab),
    Card(usize),
    Open(Card),
    Confirm,
    Tactic(Tactic),
    CardPage(i32),
    InvPage(i32),
    Item(Item),
    JournalTab(usize),
    Page(i32),
    Read(String, String),
    Receipt,
    CloseReceipt,
    Draft(usize, i32),
    Preset([u8; 4]),
    Mode,
    Sound,
    Motion,
    Brightness,
    Quit,
    Achievements,
}
pub struct Canvas<'a> {
    pub font: &'a Font,
    pub mouse: Vec2,
    pub click: bool,
    pub world: World,
}
impl<'a> Canvas<'a> {
    pub fn new(font: &'a Font, mouse: Vec2, click: bool) -> Self {
        Self {
            font,
            mouse,
            click,
            world: World::Surface,
        }
    }
    fn ink(&self) -> Color {
        if self.world == World::Surface {
            c(30, 45, 44)
        } else {
            c(233, 231, 228)
        }
    }
    fn muted(&self) -> Color {
        if self.world == World::Surface {
            c(87, 104, 102)
        } else {
            c(154, 157, 168)
        }
    }
    fn panel_color(&self) -> Color {
        if self.world == World::Surface {
            c(247, 248, 240)
        } else {
            c(32, 28, 39)
        }
    }
    fn accent(&self) -> Color {
        if self.world == World::Surface {
            c(32, 99, 105)
        } else {
            c(226, 146, 167)
        }
    }
    pub fn text(&self, t: &str, x: f32, y: f32, size: f32, color: Color) {
        draw_text_ex(
            t,
            x,
            y,
            TextParams {
                font: Some(self.font),
                font_size: size as u16,
                color,
                ..Default::default()
            },
        );
    }
    fn lines(&self, t: &str, width: f32, size: f32) -> Vec<String> {
        let mut out = vec![];
        let mut line = String::new();
        for ch in t.chars() {
            if ch == '\u{000c}' {
                if !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                }
                out.push("\u{000c}".into());
                continue;
            }
            if ch == '\n' {
                out.push(std::mem::take(&mut line));
                continue;
            }
            let trial = format!("{line}{ch}");
            if !line.is_empty()
                && measure_text(&trial, Some(self.font), size as u16, 1.).width > width
            {
                let carry = if "，。；：？！、）》」』】".contains(ch)
                    || line
                        .chars()
                        .last()
                        .is_some_and(|c| "（《「『【".contains(c))
                {
                    line.pop()
                } else {
                    None
                };
                out.push(std::mem::take(&mut line));
                if let Some(c) = carry {
                    line.push(c);
                }
            }
            line.push(ch);
        }
        if !line.is_empty() {
            out.push(line)
        }
        out
    }
    fn wrap(&self, t: &str, r: Rect, size: f32, color: Color, max: usize) -> usize {
        let lines = self.lines(t, r.w, size);
        for (i, line) in lines.iter().take(max).enumerate() {
            let mut line = line.clone();
            if i + 1 == max && lines.len() > max {
                while !line.is_empty()
                    && measure_text(&format!("{line}…"), Some(self.font), size as u16, 1.).width
                        > r.w
                {
                    line.pop();
                }
                line.push('…');
            }
            self.text(&line, r.x, r.y + size + i as f32 * size * 1.5, size, color);
        }
        lines.len()
    }
    fn panel(&self, r: Rect) {
        draw_rectangle(r.x, r.y, r.w, r.h, self.panel_color());
        draw_rectangle_lines(
            r.x,
            r.y,
            r.w,
            r.h,
            1.,
            if self.world == World::Surface {
                c(193, 206, 194)
            } else {
                c(75, 52, 71)
            },
        );
    }
    fn button(&self, r: Rect, t: &str, enabled: bool, primary: bool) -> bool {
        let hover = r.contains(self.mouse);
        let col = if primary {
            self.accent()
        } else {
            self.panel_color()
        };
        draw_rectangle(
            r.x,
            r.y,
            r.w,
            r.h,
            if enabled && hover {
                alpha(self.accent(), 0.23)
            } else {
                alpha(col, if enabled { 1. } else { 0.35 })
            },
        );
        draw_rectangle_lines(
            r.x,
            r.y,
            r.w,
            r.h,
            1.,
            if primary || hover {
                self.accent()
            } else {
                alpha(self.muted(), 0.35)
            },
        );
        let color = if primary && !hover && enabled {
            if self.world == World::Surface {
                WHITE
            } else {
                c(30, 27, 34)
            }
        } else if enabled {
            self.ink()
        } else {
            self.muted()
        };
        self.text(t, r.x + 12., r.y + r.h / 2. + 6., 17., color);
        enabled && hover && self.click
    }
    pub fn toast(&self, s: &str) {
        draw_rectangle(250., 18., 940., 80., c(41, 47, 51));
        self.wrap(s, Rect::new(272., 30., 896., 60.), 18., WHITE, 2);
    }
}
fn overlay(cv: &Canvas, r: Rect) {
    draw_rectangle(0., 0., 1440., 900., Color::new(0.02, 0.03, 0.04, 0.78));
    cv.panel(r);
}
fn footer(cv: &Canvas, left: &str) {
    cv.text(left, 24., 888., 12., cv.muted());
    cv.text("HLG 0.2", 1310., 888., 12., cv.muted());
}
pub fn draw(app: &App, cv: &mut Canvas) -> Intent {
    cv.world = app.game.as_ref().map_or(World::Surface, |g| g.world);
    if matches!(app.screen, Screen::Title | Screen::Create) {
        cv.world = World::Surface;
    }
    clear_background(if cv.world == World::Surface {
        c(228, 235, 224)
    } else {
        c(18, 17, 25)
    });
    let mut intent = match app.screen {
        Screen::Title => title(app, cv),
        Screen::Create => character(app, cv, true),
        Screen::Game => game(app, cv),
        Screen::Pause => pause(app, cv),
        Screen::Inventory => inventory(app, cv),
        Screen::Journal => journal(app, cv),
        Screen::Character => character(app, cv, false),
        Screen::Reader => reader(app, cv),
        Screen::End => ending(app, cv),
    };
    if app.receipt_open && app.screen == Screen::Game {
        intent = receipt(app, cv);
    }
    if !app.node_group.is_empty() && app.screen == Screen::Game {
        intent = node_group(app, cv);
    }
    if app.modal.is_some() {
        intent = card_modal(app, cv);
    }
    if app.animation.is_some() {
        animation(app, cv);
        return Intent::None;
    }
    intent
}
fn title(app: &App, cv: &Canvas) -> Intent {
    app.visuals.campus(World::Surface, app.archive.gamma());
    draw_rectangle(0., 0., 610., 900., Color::new(0.03, 0.07, 0.09, 0.82));
    cv.text("HLG", 64., 126., 22., c(132, 197, 204));
    cv.text("理工恐怖故事", 58., 220., 58., WHITE);
    cv.text("晚自习已经散了。", 64., 279., 23., c(209, 220, 221));
    cv.wrap("你只是回来拿一份落在教室里的作业。传达室的灯仍亮着，门边那张离校须知，却写着白天从未听过的规矩。",Rect::new(64.,318.,455.,120.),21.,c(220,227,226),4);
    let mut i = Intent::None;
    if cv.button(
        Rect::new(64., 514., 420., 58.),
        "创建角色 · 开始新循环",
        true,
        true,
    ) {
        i = Intent::New;
    }
    if cv.button(
        Rect::new(64., 590., 420., 58.),
        "继续已保存的循环",
        app.game.is_some(),
        false,
    ) {
        i = Intent::Continue;
    }
    if cv.button(Rect::new(64., 666., 205., 48.), "查看档案", true, false) {
        i = Intent::Screen(Screen::Journal);
    }
    if cv.button(Rect::new(279., 666., 205., 48.), "退出", true, false) {
        i = Intent::Quit;
    }
    cv.text(
        "属性 · 行动牌 · 支援手牌 · 掷骰",
        64.,
        771.,
        17.,
        c(159, 195, 193),
    );
    cv.text(
        "现代中国高中 / 原案故事 / 单机原生版",
        64.,
        803.,
        15.,
        c(159, 177, 179),
    );
    i
}
fn character(app: &App, cv: &Canvas, creation: bool) -> Intent {
    let values = if creation {
        app.draft
    } else {
        app.game.as_ref().map_or([3; 4], |g| g.character.values)
    };
    cv.text(
        if creation {
            "入校登记 · 创建你的角色"
        } else {
            "人物卡 · 将经历变为能力"
        },
        44.,
        66.,
        32.,
        cv.ink(),
    );
    let mut intent = Intent::None;
    if cv.button(Rect::new(1258., 28., 140., 44.), "返回", true, false) {
        intent = Intent::Back
    }
    cv.panel(Rect::new(40., 100., 416., 730.));
    app.visuals
        .student(Rect::new(60., 135., 374., 350.), 15, WHITE);
    cv.text("高三学生", 74., 527., 30., cv.ink());
    cv.wrap(
        "校服、书包和一本尚未写满的笔记。你带进校园的东西并不多，今晚能依靠的，起初只有这些。",
        Rect::new(74., 554., 340., 100.),
        20.,
        cv.muted(),
        4,
    );
    let character = Character::new(values).unwrap_or_else(|_| Character {
        values,
        xp: 0,
        growth: 0,
        level: 1,
    });
    cv.text(
        &format!(
            "生命上限 {} / 体力上限 {}",
            character.max_hp(),
            character.max_stamina()
        ),
        74.,
        708.,
        19.,
        cv.accent(),
    );
    if !creation {
        if let Some(g) = &app.game {
            cv.text(
                &format!(
                    "等级 {} · 阅历 {} · 成长点 {}",
                    g.character.level, g.character.xp, g.character.growth
                ),
                74.,
                754.,
                18.,
                cv.accent(),
            );
        }
    }
    for (j, attr) in ATTRIBUTES.into_iter().enumerate() {
        let y = 108. + j as f32 * 140.;
        cv.panel(Rect::new(488., y, 910., 124.));
        cv.text(attr.name(), 512., y + 39., 26., cv.ink());
        cv.text(&values[j].to_string(), 1140., y + 57., 40., cv.accent());
        cv.wrap(
            attr.meaning(),
            Rect::new(512., y + 55., 590., 55.),
            18.,
            cv.muted(),
            2,
        );
        if creation {
            if cv.button(
                Rect::new(1214., y + 18., 64., 42.),
                "−",
                values[j] > 1,
                false,
            ) {
                intent = Intent::Draft(j, -1)
            }
            if cv.button(
                Rect::new(1290., y + 18., 64., 42.),
                "+",
                values[j] < 5 && values.iter().map(|&v| v as u32).sum::<u32>() < 12,
                false,
            ) {
                intent = Intent::Draft(j, 1)
            }
        } else if let Some(g) = &app.game {
            if cv.button(
                Rect::new(1220., y + 36., 148., 48.),
                "成长 +1",
                g.character.growth > 0 && values[j] < 8,
                true,
            ) {
                if let Some(c) = g.legal_card(&Action::Train(attr)) {
                    intent = Intent::Open(c)
                }
            }
        }
    }
    if creation {
        let sum: u32 = values.iter().map(|&x| x as u32).sum();
        cv.text(
            &format!("分配预算：{sum} / 12    每项 1–5；先减后加即可重新分配"),
            492.,
            706.,
            20.,
            cv.ink(),
        );
        for (j, (name, stats)) in [
            ("均衡", [3, 3, 3, 3]),
            ("观察", [2, 3, 5, 2]),
            ("机动", [2, 5, 3, 2]),
            ("定心", [3, 2, 2, 5]),
        ]
        .into_iter()
        .enumerate()
        {
            if cv.button(
                Rect::new(492. + j as f32 * 126., 730., 114., 42.),
                name,
                true,
                false,
            ) {
                intent = Intent::Preset(stats)
            }
        }
        if cv.button(
            Rect::new(1036., 730., 356., 58.),
            "签好登记 · 进入校园",
            sum == 12,
            true,
        ) {
            intent = Intent::Begin
        }
        if cv.button(
            Rect::new(492., 791., 380., 40.),
            if app.easy {
                "引导模式 · 判定 +1、防御 +2"
            } else {
                "标准模式 · 使用原定判定"
            },
            true,
            false,
        ) {
            intent = Intent::Mode
        }
    }
    footer(cv, "能力数值用于本章卡牌与掷骰；成长在本局死亡后保留。");
    intent
}
pub fn map_pos(index: usize) -> Vec2 {
    let n = &NODES[index];
    vec2(885. + n.x as f32 * 61., 425. - n.y as f32 * 43.)
}
fn topbar(app: &App, cv: &Canvas, g: &Game) -> Intent {
    let mut intent = Intent::None;
    app.visuals.portrait(
        Rect::new(24., 17., 58., 64.),
        (g.hp * 100 / g.character.max_hp()).clamp(0, 100),
        g.world,
        app.clock,
        false,
        !app.archive.reduced_motion,
    );
    cv.text(g.world.name(), 99., 42., 26., cv.ink());
    cv.text(
        &format!("第 {} 回合 / 等级 {}", g.turn, g.character.level),
        100.,
        68.,
        14.,
        cv.muted(),
    );
    for (j, (name, value, max, col)) in [
        ("生命", g.hp, g.character.max_hp(), c(186, 77, 75)),
        ("体力", g.stamina, g.max_stamina(), c(72, 115, 91)),
        ("理智", g.sanity, 100, c(59, 128, 149)),
    ]
    .into_iter()
    .enumerate()
    {
        let x = 305. + j as f32 * 179.;
        cv.text(&format!("{name} {value}/{max}"), x, 41., 19., cv.ink());
        art::meter(
            Rect::new(x, 57., 156., 8.),
            value as f32 / max as f32,
            col,
            g.world,
            g.sanity,
            app.clock,
        );
    }
    cv.text(
        &format!("信物 {}/8", g.relic_count()),
        865.,
        41.,
        21.,
        cv.accent(),
    );
    let mut effects = vec![];
    if g.flag("saponified") {
        effects.push("皂化：每回合体力 -1".to_string());
    }
    if g.counter("cold_until") > g.turn && g.equip != Some(Item::Wenjin) {
        effects.push(format!(
            "寒流：移动体力 +1，剩余 {} 回合",
            g.counter("cold_until") - g.turn
        ));
    }
    if g.flag("dark_active") {
        effects.push("深渊信物：每回合理智 -2".to_string());
    }
    cv.wrap(
        &effects.join("  /  "),
        Rect::new(305., 73., 1090., 24.),
        12.,
        cv.accent(),
        1,
    );
    for (j, (label, s)) in [
        ("B 背包", Screen::Inventory),
        ("I 档案", Screen::Journal),
        ("C 人物", Screen::Character),
        ("暂停", Screen::Pause),
    ]
    .into_iter()
    .enumerate()
    {
        if cv.button(
            Rect::new(978. + j as f32 * 108., 24., 100., 48.),
            label,
            true,
            false,
        ) {
            intent = Intent::Screen(s)
        }
    }
    intent
}
fn game(app: &App, cv: &Canvas) -> Intent {
    let Some(g) = &app.game else {
        return Intent::None;
    };
    let mut intent = topbar(app, cv, g);
    cv.panel(Rect::new(24., 100., 810., 376.));
    cv.panel(Rect::new(854., 100., 562., 376.));
    app.visuals.scene(
        Rect::new(26., 102., 806., 302.),
        NODES[g.current].scene(),
        g.world,
        app.clock,
        g.in_combat(),
        app.archive.gamma(),
    );
    draw_rectangle(26., 337., 806., 67., Color::new(0.01, 0.03, 0.04, 0.74));
    cv.text(&NODES[g.current].name, 45., 380., 32., WHITE);
    let pos = g.position(g.current);
    cv.text(
        &format!("({}, {}, {})", pos[0], pos[1], pos[2]),
        637.,
        378.,
        18.,
        c(218, 228, 220),
    );
    if g.in_combat() {
        let enemies = g.active_enemies();
        let e = &g.enemies[enemies[0]];
        cv.text(
            &format!(
                "{} · 攻击 d{} → {}",
                e.kind.name(),
                e.kind.attack(),
                if e.kind.mental() { "理智" } else { "生命" }
            ),
            43.,
            435.,
            20.,
            c(190, 80, 80),
        );
        cv.text(
            &format!(
                "{}   在场 {} 个实体",
                if e.kind.mortal() {
                    format!("生命 {}", e.hp)
                } else {
                    "本章无法击败，可驱离".into()
                },
                enemies.len()
            ),
            43.,
            460.,
            15.,
            cv.muted(),
        );
    } else {
        cv.text(
            if NODES[g.current].safe {
                "这里能暂时休整，走廊中的追踪已经停下。"
            } else {
                "先核对环境和资源，再选一张行动牌。"
            },
            43.,
            438.,
            18.,
            cv.muted(),
        );
        if cv.button(Rect::new(659., 416., 155., 40.), "读场景全文", true, false) {
            intent = Intent::Read(
                NODES[g.current].name.clone(),
                NODES[g.current].description(g.world),
            );
        }
    }
    if let Some(m) = draw_map(app, cv, g) {
        intent = m;
    }
    cv.panel(Rect::new(24., 489., 810., 119.));
    cv.panel(Rect::new(854., 489., 562., 119.));
    if let Some(r) = &app.receipt {
        cv.text(&r.title, 43., 519., 21., cv.accent());
        cv.wrap(
            &r.changes
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join("；"),
            Rect::new(43., 533., 604., 67.),
            16.,
            cv.ink(),
            2,
        );
        if cv.button(Rect::new(665., 536., 150., 46.), "V 结果详情", true, false) {
            intent = Intent::Receipt
        }
    }
    let guide = feedback::guide(g);
    cv.text(&guide.title, 872., 518., 19., cv.accent());
    cv.wrap(
        &guide.detail,
        Rect::new(872., 530., 522., 66.),
        15.,
        cv.ink(),
        3,
    );
    for (j, (name, tab)) in [
        ("现场事件", CardTab::Explore),
        ("移动方式", CardTab::Move),
        ("战斗应对", CardTab::Combat),
        ("道具与制作", CardTab::Items),
    ]
    .into_iter()
    .enumerate()
    {
        if cv.button(
            Rect::new(24. + j as f32 * 150., 623., 140., 43.),
            name,
            true,
            app.tab == tab,
        ) {
            intent = Intent::Tab(tab)
        }
    }
    cv.text("手牌", 657., 651., 16., cv.muted());
    for (j, t) in g.hand.iter().enumerate() {
        let x = 712. + j as f32 * 190.;
        cv.panel(Rect::new(x, 623., 177., 43.));
        cv.text(t.name(), x + 12., 650., 17., cv.accent());
    }
    let cards = app.visible_cards();
    let pages = cards.len().div_ceil(5).max(1);
    let page = app.card_page.min(pages - 1);
    if cv.button(Rect::new(1292., 623., 45., 43.), "‹", page > 0, false) {
        intent = Intent::CardPage(-1)
    }
    if cv.button(
        Rect::new(1347., 623., 45., 43.),
        "›",
        page + 1 < pages,
        false,
    ) {
        intent = Intent::CardPage(1)
    }
    for (j, card) in cards.iter().enumerate().skip(page * 5).take(5) {
        let x = 24. + (j % 5) as f32 * 280.;
        let r = Rect::new(x, 679., 270., 188.);
        cv.panel(r);
        draw_rectangle(
            x,
            679.,
            270.,
            4.,
            if card.enabled {
                cv.accent()
            } else {
                cv.muted()
            },
        );
        if let Some(item) = card.icon {
            app.visuals.item(
                Rect::new(x + 187., 704., 64., 64.),
                item.info().icon,
                if card.enabled {
                    WHITE
                } else {
                    alpha(WHITE, 0.4)
                },
            );
        }
        cv.text(
            &format!("{}  {}", j % 5 + 1, card.title),
            x + 14.,
            711.,
            17.,
            if card.enabled { cv.ink() } else { cv.muted() },
        );
        cv.wrap(
            &card.cost,
            Rect::new(x + 14., 728., 170., 44.),
            13.,
            cv.accent(),
            2,
        );
        cv.wrap(
            if card.enabled {
                &card.rule
            } else {
                &card.reason
            },
            Rect::new(x + 14., 788., 242., 62.),
            14.,
            cv.muted(),
            3,
        );
        if r.contains(cv.mouse) {
            draw_rectangle_lines(x - 1., 678., 272., 190., 2., cv.accent());
            if cv.click {
                intent = Intent::Card(j)
            }
        }
    }
    if cards.is_empty() {
        cv.text(
            if app.selected == Some(g.current) {
                "这里就是当前位置；请选择一个相邻地点。"
            } else if app.selected.is_some() {
                "目标不相邻；先沿连线前进，或持握适用的移动信物。"
            } else {
                "先在地图选择目标地点，再查看可用的移动卡。"
            },
            48.,
            756.,
            22.,
            cv.muted(),
        );
    }
    footer(
        cv,
        if app.save_error {
            "保存失败；请查看顶部提示。"
        } else {
            "已自动保存 · 选牌和阅读不推进回合 · Esc 返回/暂停"
        },
    );
    intent
}
fn draw_map(app: &App, cv: &Canvas, g: &Game) -> Option<Intent> {
    if g.world == World::Echo {
        return echo_map(app, cv, g);
    }
    let mut result = None;
    for z in 0..=5 {
        if cv.button(
            Rect::new(870. + z as f32 * 86., 113., 78., 35.),
            &if z == 0 {
                "地下".into()
            } else {
                format!("{z} 层")
            },
            true,
            app.floor == z,
        ) {
            result = Some(Intent::Floor(z));
        }
    }
    let selected = app.selected.unwrap_or(g.current);
    cv.text(
        &format!(
            "{}  ({},{},{})",
            NODES[selected].name, NODES[selected].x, NODES[selected].y, NODES[selected].z
        ),
        871.,
        174.,
        17.,
        cv.ink(),
    );
    for x in 0..=8 {
        draw_line(
            885. + x as f32 * 61.,
            192.,
            885. + x as f32 * 61.,
            431.,
            1.,
            alpha(cv.muted(), 0.09),
        );
    }
    for y in 0..=5 {
        draw_line(
            880.,
            425. - y as f32 * 43.,
            1382.,
            425. - y as f32 * 43.,
            1.,
            alpha(cv.muted(), 0.09),
        );
    }
    for e in EDGES.iter() {
        let a = &NODES[e.a];
        let b = &NODES[e.b];
        if e.worlds & g.world.mask() == 0
            || a.z != app.floor
            || b.z != app.floor
            || !g.known[g.world.index()].contains(&e.a)
            || !g.known[g.world.index()].contains(&e.b)
        {
            continue;
        }
        let p = map_pos(e.a);
        let q = map_pos(e.b);
        draw_line(p.x, p.y, q.x, q.y, 2., alpha(cv.accent(), 0.26));
    }
    let mut groups = std::collections::BTreeMap::<(i32, i32), Vec<usize>>::new();
    for (i, n) in NODES.iter().enumerate() {
        if n.z == app.floor && n.exists(g.world) && g.known[g.world.index()].contains(&i) {
            groups.entry((n.x, n.y)).or_default().push(i);
        }
    }
    for ids in groups.values() {
        let p = map_pos(ids[0]);
        let current = ids.contains(&g.current);
        let selected = app.selected.is_some_and(|s| ids.contains(&s));
        let col = if current {
            cv.accent()
        } else if selected {
            c(182, 132, 52)
        } else {
            cv.muted()
        };
        draw_circle(p.x, p.y, if current { 9. } else { 6. }, col);
        if current || selected {
            draw_circle_lines(p.x, p.y, 14., 2., col);
        }
        if ids.len() > 1 {
            cv.text(&format!("{}", ids.len()), p.x + 8., p.y - 7., 11., col);
        }
        let name = if ids.len() > 1 {
            "多个房间".into()
        } else {
            NODES[ids[0]].name.clone()
        };
        let chars: Vec<char> = name.chars().collect();
        for (line, ch) in chars.chunks(5).take(2).enumerate() {
            let text: String = ch.iter().collect();
            let w = measure_text(&text, Some(cv.font), 10, 1.).width;
            cv.text(
                &text,
                p.x - w / 2.,
                p.y + 20. + line as f32 * 11.,
                10.,
                cv.ink(),
            );
        }
        let r = Rect::new(p.x - 25., p.y - 14., 50., 40.);
        if r.contains(cv.mouse) {
            draw_circle_lines(p.x, p.y, 17., 1., cv.accent());
            if cv.click {
                result = Some(if ids.len() == 1 {
                    Intent::Select(ids[0])
                } else {
                    Intent::Group(ids.clone())
                });
            }
        }
    }
    cv.text(
        "实心点为当前位置；重叠坐标可点开选择房间。",
        873.,
        466.,
        12.,
        cv.muted(),
    );
    result
}
fn echo_map(app: &App, cv: &Canvas, g: &Game) -> Option<Intent> {
    let mut result = None;
    let z = if app.floor >= 0 {
        app.floor as usize % hlg_core::echo::ZONES
    } else {
        g.echo.zone(g.current)
    };
    if cv.button(
        Rect::new(870., 113., 154., 35.),
        "区段总览",
        true,
        app.floor < 0,
    ) {
        result = Some(Intent::Floor(-1))
    }
    if cv.button(Rect::new(1036., 113., 80., 35.), "‹", z > 0, false) {
        result = Some(Intent::Floor(z as i32 - 1))
    }
    if cv.button(
        Rect::new(1124., 113., 166., 35.),
        &format!("第 {} 区段", z + 1),
        true,
        app.floor == g.map_index(),
    ) {
        result = Some(Intent::Floor(g.map_index()))
    }
    if cv.button(
        Rect::new(1300., 113., 80., 35.),
        "›",
        z + 1 < hlg_core::echo::ZONES,
        false,
    ) {
        result = Some(Intent::Floor(z as i32 + 1))
    }
    cv.text(
        &format!(
            "{} 个房间 · 第 {} 次重组 · 种子 {}",
            NODES.len(),
            g.echo.revision,
            g.seed
        ),
        872.,
        173.,
        15.,
        cv.accent(),
    );
    if app.floor < 0 {
        let goal = feedback::guide(g);
        let goal_zone = goal
            .destination
            .filter(|_| goal.world == Some(World::Echo))
            .map(|n| g.echo.zone(n));
        for region in 0..hlg_core::echo::ZONES {
            let x = 879. + (region % 4) as f32 * 128.;
            let y = 191. + (region / 4) as f32 * 60.;
            let r = Rect::new(x, y, 119., 53.);
            cv.panel(r);
            let count = g.known[1]
                .iter()
                .filter(|&&n| g.echo.zone(n) == region)
                .count();
            cv.text(
                &format!(
                    "{} 区 {}",
                    region + 1,
                    if region == g.echo.zone(g.current) {
                        "当前位置"
                    } else if goal_zone == Some(region) {
                        "目标"
                    } else {
                        ""
                    }
                ),
                x + 8.,
                y + 21.,
                14.,
                if region == g.echo.zone(g.current) {
                    cv.accent()
                } else {
                    cv.ink()
                },
            );
            cv.text(
                &format!("已核实 {count} 处"),
                x + 8.,
                y + 42.,
                12.,
                cv.muted(),
            );
            if r.contains(cv.mouse) && cv.click {
                result = Some(Intent::Floor(region as i32))
            }
        }
        cv.text(
            "上下相邻为不同深度；左右相邻为同层区段。",
            873.,
            466.,
            12.,
            cv.muted(),
        );
        return result;
    }
    let map_point = |id: usize| {
        let p = g.echo.position(id);
        vec2(
            887. + (p[0] - (z % 4 * 10) as i32) as f32 * 70.,
            418. - p[1] as f32 * 29.,
        )
    };
    for id in 0..NODES.len() {
        if g.echo.zone(id) != z || !g.known[1].contains(&id) {
            continue;
        }
        for b in g.echo.neighbors(id) {
            if g.echo.zone(b) == z && b > id && g.known[1].contains(&b) {
                let p = map_point(id);
                let q = map_point(b);
                draw_line(p.x, p.y, q.x, q.y, 2., alpha(cv.accent(), 0.35));
            }
        }
    }
    let mut groups = std::collections::BTreeMap::<[i32; 3], Vec<usize>>::new();
    for id in 0..NODES.len() {
        if g.echo.zone(id) == z && g.known[1].contains(&id) {
            groups.entry(g.echo.position(id)).or_default().push(id);
        }
    }
    for ids in groups.values() {
        let p = map_point(ids[0]);
        let current = ids.contains(&g.current);
        let selected = app.selected.is_some_and(|id| ids.contains(&id));
        let col = if current {
            cv.accent()
        } else if selected {
            c(212, 165, 81)
        } else {
            cv.muted()
        };
        draw_circle(p.x, p.y, if current { 7. } else { 4. }, col);
        if current || selected {
            draw_circle_lines(p.x, p.y, 12., 2., col)
        }
        let anchor = ids.iter().copied().find(|&n| n < *SOURCE_NODE_COUNT);
        let special = ids
            .iter()
            .copied()
            .find(|&n| NODES[n].mirror || NODES[n].safe || NODES[n].roof());
        let exit = ids
            .iter()
            .copied()
            .find(|&n| g.echo.neighbors(n).iter().any(|&b| g.echo.zone(b) != z));
        let label = if let Some(n) = anchor {
            Some(NODES[n].name.chars().take(5).collect::<String>())
        } else if let Some(n) = special {
            Some(
                if NODES[n].safe {
                    "休整"
                } else if NODES[n].mirror {
                    "镜面"
                } else {
                    "楼顶"
                }
                .into(),
            )
        } else if exit.is_some() {
            Some("区段门".into())
        } else {
            None
        };
        if let Some(label) = label {
            cv.text(&label, p.x - 22., p.y + 15., 10., cv.ink());
        }
        if Rect::new(p.x - 16., p.y - 12., 48., 27.).contains(cv.mouse) && cv.click {
            result = Some(if ids.len() > 1 {
                Intent::Group(ids.clone())
            } else {
                Intent::Select(ids[0])
            });
        }
    }
    let selected = app.selected.unwrap_or(g.current);
    cv.text(
        &format!(
            "{} · {}",
            NODES[selected].name,
            if selected < *SOURCE_NODE_COUNT {
                "剧情房间"
            } else {
                "复制房间"
            }
        ),
        873.,
        447.,
        14.,
        cv.ink(),
    );
    if let Some(next) = g
        .neighbors(g.current)
        .into_iter()
        .find(|&n| g.echo.zone(n) != g.echo.zone(g.current))
    {
        if cv.button(
            Rect::new(1174., 429., 220., 29.),
            &format!("跨入第 {} 区 →", g.echo.zone(next) + 1),
            true,
            true,
        ) {
            result = Some(Intent::Select(next));
        }
    }
    cv.text(
        "逐处查看可核实全区；区段门通往邻区；M 返回当前位置。",
        873.,
        467.,
        12.,
        cv.muted(),
    );
    result
}
fn node_group(app: &App, cv: &Canvas) -> Intent {
    overlay(cv, Rect::new(440., 170., 560., 560.));
    cv.text("这个坐标有多个房间", 468., 218., 26., cv.ink());
    let mut result = Intent::None;
    for (j, &id) in app.node_group.iter().enumerate() {
        if cv.button(
            Rect::new(468., 250. + j as f32 * 62., 504., 52.),
            &format!(
                "{}{}",
                NODES[id].name,
                if NODES[id].interior { " · 内景" } else { "" }
            ),
            true,
            false,
        ) {
            result = Intent::Select(id)
        }
    }
    if cv.button(Rect::new(468., 660., 504., 44.), "返回地图", true, false) {
        result = Intent::Back
    }
    result
}
fn card_modal(app: &App, cv: &Canvas) -> Intent {
    let card = app.modal.as_ref().unwrap();
    let Some(g) = &app.game else {
        return Intent::None;
    };
    overlay(cv, Rect::new(164., 82., 1112., 744.));
    cv.text("行动预览", 194., 122., 16., cv.muted());
    cv.text(&card.title, 194., 168., 32., cv.ink());
    if let Some(i) = card.icon {
        app.visuals
            .item(Rect::new(1041., 116., 178., 178.), i.info().icon, WHITE);
    }
    cv.wrap(
        &card.story,
        Rect::new(194., 190., 772., 150.),
        21.,
        cv.ink(),
        5,
    );
    cv.text("消耗与结果", 194., 359., 18., cv.accent());
    cv.wrap(
        &card.cost,
        Rect::new(194., 375., 1006., 70.),
        21.,
        cv.ink(),
        2,
    );
    cv.wrap(
        &card.rule,
        Rect::new(194., 442., 1008., 105.),
        18.,
        cv.ink(),
        3,
    );
    if !card.enabled {
        cv.wrap(
            &card.reason,
            Rect::new(194., 552., 1002., 60.),
            19.,
            c(188, 79, 74),
            2,
        );
    }
    let support_allowed = !matches!(card.action, Action::Equip(_) | Action::Train(_));
    let mut result = Intent::None;
    cv.text(
        if support_allowed {
            "可投入一张手牌；用后弃置并补回一张。"
        } else {
            "整理装备或成长不使用手牌，也不推进回合。"
        },
        194.,
        588.,
        16.,
        cv.muted(),
    );
    for (j, &t) in g.hand.iter().enumerate() {
        let r = Rect::new(194. + j as f32 * 340., 608., 324., 88.);
        cv.panel(r);
        if app.tactic == Some(t) {
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 3., cv.accent());
        }
        cv.text(t.name(), r.x + 13., r.y + 29., 20., cv.accent());
        cv.wrap(
            t.effect(),
            Rect::new(r.x + 13., r.y + 40., 294., 40.),
            14.,
            cv.muted(),
            2,
        );
        if r.contains(cv.mouse) && cv.click && support_allowed {
            result = Intent::Tactic(t)
        }
    }
    cv.text(
        &format!(
            "体魄 {} / 机敏 {} / 洞察 {} / 定力 {}",
            g.character.values[0],
            g.character.values[1],
            g.character.values[2],
            g.character.values[3]
        ),
        194.,
        726.,
        16.,
        cv.muted(),
    );
    if let Some(t) = app.tactic {
        if let Action::Travel {
            mode: TravelMode::Walk | TravelMode::Careful,
            ..
        } = card.action
        {
            if t != Tactic::Quiet && t != Tactic::Guard {
                cv.text(
                    "此牌不增加移动检定，仍会消耗并补牌。",
                    759.,
                    726.,
                    13.,
                    cv.accent(),
                );
            }
        }
    }
    if cv.button(
        Rect::new(194., 753., 218., 48.),
        "返回，暂不行动",
        true,
        false,
    ) {
        result = Intent::Back
    }
    if cv.button(
        Rect::new(859., 746., 383., 55.),
        "确认出卡 · 执行动画与结算",
        card.enabled,
        true,
    ) {
        result = Intent::Confirm
    }
    result
}
fn receipt(app: &App, cv: &Canvas) -> Intent {
    let Some(r) = &app.receipt else {
        return Intent::CloseReceipt;
    };
    overlay(cv, Rect::new(202., 55., 1036., 790.));
    cv.text(&r.title, 235., 107., 29., cv.ink());
    let mut text = String::from("本次变化\n");
    text.push_str(&r.changes.join("\n"));
    if !r.rolls.is_empty() {
        text.push_str("\n\n判定过程\n");
        for roll in &r.rolls {
            text.push_str(&if roll.defense_sides > 0 {
                format!(
                    "{}：攻击 d{} = {}，防御 d{} = {}，正差值 {}。\n",
                    roll.label,
                    roll.sides,
                    roll.value,
                    roll.defense_sides,
                    roll.defense,
                    (roll.value - roll.defense).max(0)
                )
            } else {
                format!(
                    "{}：d{} = {}，修正 {:+}，合计 {} / 目标 {}，{}。\n",
                    roll.label,
                    roll.sides,
                    roll.value,
                    roll.bonus,
                    roll.value + roll.bonus,
                    roll.target,
                    if roll.success { "通过" } else { "未通过" }
                )
            });
        }
    }
    text.push_str("\n发生了什么\n");
    text.push_str(&r.story);
    text.push_str("\n\n需要关注\n");
    text.push_str(&r.notes.join("\n"));
    let pages = paginate(cv, &text, 960., 18., 20);
    let page = app.receipt_page.min(pages.len() - 1);
    for (j, line) in pages[page].iter().enumerate() {
        cv.text(line, 235., 154. + j as f32 * 28., 18., cv.ink());
    }
    let mut i = Intent::None;
    cv.text(
        &format!("第 {} / {} 页", page + 1, pages.len()),
        628.,
        792.,
        16.,
        cv.muted(),
    );
    if cv.button(Rect::new(235., 762., 130., 48.), "上一页", page > 0, false) {
        i = Intent::Page(-1)
    }
    if cv.button(
        Rect::new(379., 762., 130., 48.),
        "下一页",
        page + 1 < pages.len(),
        false,
    ) {
        i = Intent::Page(1)
    }
    if cv.button(
        Rect::new(918., 762., 285., 48.),
        "我知道了 · 继续",
        true,
        true,
    ) {
        i = Intent::CloseReceipt
    }
    i
}
fn paginate(cv: &Canvas, text: &str, width: f32, size: f32, height: usize) -> Vec<Vec<String>> {
    let mut pages = vec![];
    let mut page = vec![];
    for line in cv.lines(text, width, size) {
        if line == "\u{000c}" {
            if !page.is_empty() {
                pages.push(std::mem::take(&mut page));
            }
            continue;
        }
        if page.len() == height {
            pages.push(std::mem::take(&mut page));
        }
        page.push(line);
    }
    if !page.is_empty() {
        pages.push(page)
    }
    if pages.is_empty() {
        pages.push(vec![])
    }
    pages
}
fn animation(app: &App, cv: &Canvas) {
    let anim = app.animation.as_ref().unwrap();
    let elapsed = (get_time() - anim.started) as f32;
    let p = (elapsed / 1.65).clamp(0., 1.);
    overlay(cv, Rect::new(144., 83., 1152., 728.));
    let g = app.game.as_ref().unwrap();
    let shown = if p < 0.45 { &anim.before } else { g };
    app.visuals.scene(
        Rect::new(164., 105., 1112., 445.),
        NODES[shown.current].scene(),
        shown.world,
        app.clock,
        shown.in_combat(),
        app.archive.gamma(),
    );
    draw_rectangle(164., 105., 1112., 445., Color::new(0.02, 0.03, 0.04, 0.23));
    let frame = if app.archive.reduced_motion {
        15
    } else {
        match anim.motion.as_str() {
            "walk" => (elapsed * 8.) as usize % 4,
            "explore" => 4 + (elapsed * 5.) as usize % 4,
            "guard" => 8 + (elapsed * 4.) as usize % 2,
            "combat" => {
                if p < 0.5 {
                    10
                } else if g.hp < anim.before.hp {
                    11
                } else {
                    15
                }
            }
            "item" | "craft" | "equip" => {
                if p < 0.6 {
                    12
                } else {
                    15
                }
            }
            "mirror" | "rearrange" => {
                if p < 0.6 {
                    14
                } else {
                    15
                }
            }
            _ => 15,
        }
    };
    let x = if anim.motion == "walk" && !app.archive.reduced_motion {
        225. + p * 300.
    } else {
        245.
    };
    app.visuals
        .student(Rect::new(x, 224., 450., 300.), frame, WHITE);
    if let Some(e) = anim
        .before
        .active_enemies()
        .first()
        .map(|&j| &anim.before.enemies[j])
    {
        app.visuals.enemy(
            Rect::new(880., 212., 220., 320.),
            e.kind,
            if p > 0.55 { alpha(WHITE, 0.78) } else { WHITE },
        );
        cv.text(e.kind.name(), 872., 536., 20., WHITE);
    }
    if let Action::Use(i) | Action::Craft(i) | Action::Equip(Some(i)) = &anim.action {
        app.visuals.item(
            Rect::new(710., 260. - p * 25., 140., 140.),
            i.info().icon,
            WHITE,
        );
    }
    if anim.motion == "combat" && p > 0.35 && p < 0.8 {
        let xx = 630. + (p - 0.35) * 520.;
        draw_circle(xx, 360., 12., alpha(c(249, 193, 91), 0.8));
        draw_line(628., 360., xx, 360., 4., c(239, 193, 106));
    }
    if anim.motion == "mirror"
        || anim.motion == "rearrange"
        || g.echo.revision != anim.before.echo.revision
    {
        for j in 0..12 {
            let x = 164. + j as f32 * 94.;
            draw_rectangle(
                x,
                105. + (j as f32 * 11. + p * 90.).sin() * 8.,
                4.,
                445.,
                alpha(c(176, 77, 133), 0.2),
            );
        }
        if anim.resolved {
            cv.text("门牌与通路正在重新排列", 805., 161., 22., WHITE);
        }
    }
    cv.text(&anim.label, 186., 596., 31., cv.ink());
    if anim.resolved {
        if let Some(r) = g.last.rolls.first() {
            let rolling = elapsed < 1.15 && !app.archive.reduced_motion;
            let value = if rolling {
                1 + ((elapsed * 27.) as i32 % r.sides.max(1))
            } else {
                r.value
            };
            cv.panel(Rect::new(192., 625., 88., 88.));
            cv.text(&value.to_string(), 214., 685., 44., cv.accent());
            cv.text(
                &format!("{} · d{}", r.label, r.sides),
                301.,
                652.,
                20.,
                cv.ink(),
            );
            cv.text(
                &if rolling {
                    "掷骰中，停下后显示算式…".into()
                } else if r.defense_sides > 0 {
                    format!(
                        "攻击 {} / 防御 {} / 骰差 {}",
                        r.value,
                        r.defense,
                        (r.value - r.defense).max(0)
                    )
                } else {
                    format!(
                        "{} {:+} = {}   目标 {}   {}",
                        r.value,
                        r.bonus,
                        r.value + r.bonus,
                        r.target,
                        if r.success { "通过" } else { "未通过" }
                    )
                },
                301.,
                692.,
                24.,
                cv.accent(),
            );
        } else {
            cv.text("确定行动已执行 · 无随机判定", 192., 668., 24., cv.accent());
        }
        if let Some(r) = &app.receipt {
            cv.wrap(
                &r.changes
                    .iter()
                    .take(3)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("；"),
                Rect::new(192., 735., 1054., 46.),
                17.,
                cv.ink(),
                2,
            );
        }
    } else {
        cv.text("正在执行所选行动…", 192., 669., 24., cv.muted());
    }
}
fn inventory(app: &App, cv: &Canvas) -> Intent {
    let Some(g) = &app.game else {
        return Intent::Back;
    };
    let mut result = Intent::None;
    cv.text("背包 · 先查看用途，再决定使用", 40., 65., 30., cv.ink());
    cv.text(
        &format!("当前持握：{}", g.equip.map_or("双手空闲", |i| i.name())),
        40.,
        102.,
        18.,
        cv.accent(),
    );
    if cv.button(Rect::new(1265., 28., 130., 45.), "返回", true, false) {
        result = Intent::Back
    }
    let owned: Vec<(Item, u16)> = g.inv.iter().map(|(&i, &n)| (i, n)).collect();
    let pages = owned.len().div_ceil(20).max(1);
    let page = app.inv_page.min(pages - 1);
    for (j, &(item, count)) in owned.iter().enumerate().skip(page * 20).take(20) {
        let x = 40. + (j % 5) as f32 * 163.;
        let y = 129. + ((j % 20) / 5) as f32 * 163.;
        let r = Rect::new(x, y, 151., 151.);
        cv.panel(r);
        if app.item == Some(item) {
            draw_rectangle_lines(x, y, 151., 151., 3., cv.accent());
        }
        app.visuals.item(
            Rect::new(x + 39., y + 9., 76., 76.),
            item.info().icon,
            WHITE,
        );
        cv.text(item.name(), x + 9., y + 112., 15., cv.ink());
        cv.text(
            &format!("{} · ×{}", item.info().kind, count),
            x + 9.,
            y + 136.,
            12.,
            cv.muted(),
        );
        if r.contains(cv.mouse) && cv.click {
            result = Intent::Item(item)
        }
    }
    if cv.button(Rect::new(40., 803., 120., 44.), "上一页", page > 0, false) {
        result = Intent::InvPage(-1)
    }
    if cv.button(
        Rect::new(173., 803., 120., 44.),
        "下一页",
        page + 1 < pages,
        false,
    ) {
        result = Intent::InvPage(1)
    }
    cv.text(
        &format!("{} / {} 页", page + 1, pages),
        330.,
        831.,
        17.,
        cv.muted(),
    );
    cv.panel(Rect::new(883., 129., 512., 718.));
    if let Some(item) = app
        .item
        .filter(|i| g.has(*i))
        .or_else(|| owned.first().map(|(i, _)| *i))
    {
        app.visuals
            .item(Rect::new(1227., 145., 140., 140.), item.info().icon, WHITE);
        cv.text(item.name(), 906., 177., 27., cv.ink());
        cv.text(&item.info().kind, 906., 210., 17., cv.accent());
        cv.wrap(
            item.description(),
            Rect::new(906., 295., 464., 174.),
            18.,
            cv.ink(),
            6,
        );
        cv.text("实际用途", 906., 503., 19., cv.accent());
        cv.wrap(
            item.effect(),
            Rect::new(906., 515., 462., 90.),
            17.,
            cv.ink(),
            3,
        );
        let mut cards = g.item_cards(item);
        if item == Item::Card {
            cards = g
                .quest_cards()
                .into_iter()
                .filter(|c| c.action == Action::Quest("wenjin".into()))
                .collect();
        }
        if item == Item::Fuel {
            cv.wrap(
                "持握骨轮，在里世界楼顶选择另一处楼顶，飞行卡会出现。五层室内地点不算楼顶。",
                Rect::new(906., 627., 459., 84.),
                18.,
                cv.muted(),
                3,
            );
        } else if item == Item::Tear {
            cv.wrap(
                &format!(
                    "当前收集 {}/7。作为结局条件携带，无需点击消耗。",
                    g.count(item)
                ),
                Rect::new(906., 627., 459., 84.),
                18.,
                cv.muted(),
                3,
            );
        } else if cards.is_empty() {
            cv.wrap(
                "此物由场景条件自动识别，不会在背包中盲目消耗。到对应地点后查看事件卡。",
                Rect::new(906., 627., 459., 100.),
                18.,
                cv.muted(),
                4,
            );
        } else {
            for (j, card) in cards.iter().take(3).enumerate() {
                if cv.button(
                    Rect::new(906., 625. + j as f32 * 61., 464., 52.),
                    &card.title,
                    true,
                    j == 0,
                ) {
                    result = Intent::Open(card.clone());
                }
            }
        }
        if g.equip.is_some() && cv.button(Rect::new(611., 803., 225., 44.), "腾出双手", true, false)
        {
            if let Some(c) = g.legal_card(&Action::Equip(None)) {
                result = Intent::Open(c)
            }
        }
    }
    footer(
        cv,
        "背包检查、翻页和阅读不推进回合。持握每次只保留一件，材料与权限只需携带。",
    );
    result
}
fn reader(app: &App, cv: &Canvas) -> Intent {
    cv.text(&app.reader_title, 60., 74., 31., cv.ink());
    let mut result = Intent::None;
    cv.panel(Rect::new(60., 110., 1320., 695.));
    let pages = paginate(cv, &app.reader_text, 1192., 21., 19);
    let page = app.reader_page.min(pages.len() - 1);
    for (j, line) in pages[page].iter().enumerate() {
        cv.text(line, 105., 155. + j as f32 * 33., 21., cv.ink());
    }
    if cv.button(Rect::new(60., 822., 160., 48.), "上一页", page > 0, false) {
        result = Intent::Page(-1)
    }
    if cv.button(
        Rect::new(236., 822., 160., 48.),
        "下一页",
        page + 1 < pages.len(),
        false,
    ) {
        result = Intent::Page(1)
    }
    cv.text(
        &format!("{} / {} 页 · 阅读不推进回合", page + 1, pages.len()),
        525.,
        854.,
        17.,
        cv.muted(),
    );
    if cv.button(Rect::new(1160., 822., 220., 48.), "合上，返回", true, true) {
        result = Intent::Back
    }
    result
}
fn journal(app: &App, cv: &Canvas) -> Intent {
    cv.text("校园档案", 40., 67., 31., cv.ink());
    let mut result = Intent::None;
    if cv.button(Rect::new(1265., 28., 130., 44.), "返回", true, false) {
        result = Intent::Back
    }
    for (j, label) in ["收录手记", "成就目录", "行动记录", "操作说明"]
        .into_iter()
        .enumerate()
    {
        if cv.button(
            Rect::new(40. + j as f32 * 190., 98., 178., 45.),
            label,
            true,
            app.journal_tab == j,
        ) {
            result = Intent::JournalTab(j)
        }
    }
    if cv.button(
        Rect::new(1040., 98., 355., 45.),
        "原稿目录 · 含未解锁剧透",
        true,
        app.journal_tab == 4,
    ) {
        result = Intent::JournalTab(4);
    }
    match app.journal_tab {
        0 => {
            let collected: Vec<_> = DOCUMENTS
                .iter()
                .filter(|d| {
                    app.archive.clues.contains(&d.id)
                        || app.game.as_ref().is_some_and(|g| g.clues.contains(&d.id))
                })
                .collect();
            let page = app.journal_page.min(collected.len().div_ceil(6).max(1) - 1);
            for (j, d) in collected.iter().enumerate().skip(page * 6).take(6) {
                let y = 169. + (j % 6) as f32 * 104.;
                cv.panel(Rect::new(40., y, 1355., 92.));
                cv.text(&d.title, 62., y + 34., 24., cv.ink());
                cv.wrap(
                    &d.text.replace('\u{000c}', ""),
                    Rect::new(62., y + 46., 1080., 32.),
                    16.,
                    cv.muted(),
                    1,
                );
                if cv.button(Rect::new(1190., y + 22., 180., 47.), "打开全文", true, true) {
                    result = Intent::Read(d.title.clone(), d.text.clone());
                }
            }
            if collected.is_empty() {
                cv.text(
                    "暂时没有收录的手记。探索现场，找到纸页后会自动归档。",
                    65.,
                    229.,
                    22.,
                    cv.muted(),
                );
            }
            if cv.button(Rect::new(40., 817., 130., 45.), "上一页", page > 0, false) {
                result = Intent::Page(-1)
            }
            if cv.button(
                Rect::new(184., 817., 130., 45.),
                "下一页",
                (page + 1) * 6 < collected.len(),
                false,
            ) {
                result = Intent::Page(1)
            }
        }
        1 | 4 => {
            let full = app.journal_tab == 4;
            let unlocked = |name: &str| {
                app.archive.achievements.contains(name)
                    || app
                        .game
                        .as_ref()
                        .is_some_and(|g| g.achievements.contains(name))
            };
            let items: Vec<_> = ACHIEVEMENTS
                .iter()
                .filter(|a| {
                    !a.root
                        && (full
                            || unlocked(&a.name)
                            || (a.kind == "main"
                                && ACHIEVEMENTS
                                    .iter()
                                    .any(|p| a.parent.contains(&p.name) && unlocked(&p.name))))
                })
                .collect();
            let page = app.journal_page.min(items.len().div_ceil(7).max(1) - 1);
            cv.text(
                if full {
                    "原稿目录含全部成就原文与未开放条目，阅读不解锁成就。"
                } else {
                    "只显示已点亮及其相邻主线条目；未点亮时隐藏描述，彩蛋保持隐藏。"
                },
                42.,
                171.,
                16.,
                cv.muted(),
            );
            for (j, a) in items.iter().enumerate().skip(page * 7).take(7) {
                let unlocked = app.archive.achievements.contains(&a.name)
                    || app
                        .game
                        .as_ref()
                        .is_some_and(|g| g.achievements.contains(&a.name));
                let supported = achievement_supported(&a.name);
                let y = 192. + (j % 7) as f32 * 82.;
                cv.panel(Rect::new(40., y, 1355., 72.));
                draw_circle(
                    60.,
                    y + 24.,
                    5.,
                    if unlocked { cv.accent() } else { cv.muted() },
                );
                cv.text(&a.name, 80., y + 31., 21., cv.ink());
                cv.text(
                    if unlocked {
                        "已点亮"
                    } else if supported {
                        "尚未达成"
                    } else {
                        "本章未开放"
                    },
                    435.,
                    y + 29.,
                    15.,
                    cv.accent(),
                );
                cv.wrap(
                    if unlocked { &a.text } else { &a.condition },
                    Rect::new(580., y + 10., 577., 53.),
                    14.,
                    cv.muted(),
                    2,
                );
                if cv.button(
                    Rect::new(1210., y + 15., 165., 42.),
                    if unlocked {
                        "阅读成就"
                    } else if full {
                        "查看原稿"
                    } else {
                        "查看条件"
                    },
                    true,
                    false,
                ) {
                    let text = format!(
                        "{}\n\n{}\n\n达成条件：{}\n\n前置成就：{}\n\n{}{}",
                        a.name,
                        if unlocked || full {
                            a.text.as_str()
                        } else {
                            "点亮后显示成就描述。"
                        },
                        a.condition,
                        a.parent,
                        if a.placeholder && (unlocked || full) {
                            "原稿此处保留了未完成文字，未另行补写。\n"
                        } else {
                            ""
                        },
                        if !supported {
                            "本章尚未开放该条目的完整触发链。"
                        } else {
                            ""
                        }
                    );
                    result = Intent::Read(a.name.clone(), text);
                }
            }
            if cv.button(Rect::new(40., 810., 130., 45.), "上一页", page > 0, false) {
                result = Intent::Page(-1)
            }
            if cv.button(
                Rect::new(184., 810., 130., 45.),
                "下一页",
                (page + 1) * 7 < items.len(),
                false,
            ) {
                result = Intent::Page(1)
            }
            cv.text(
                &format!(
                    "当前可见 {} 项 · 第 {} / {} 页",
                    items.len(),
                    page + 1,
                    items.len().div_ceil(7).max(1)
                ),
                345.,
                840.,
                17.,
                cv.muted(),
            );
        }
        2 => {
            if let Some(g) = &app.game {
                let all = g.log.iter().rev().cloned().collect::<Vec<_>>().join("\n\n");
                let pages = paginate(cv, &all, 1300., 19., 20);
                let page = app.journal_page.min(pages.len() - 1);
                for (j, line) in pages[page].iter().enumerate() {
                    cv.text(line, 50., 193. + j as f32 * 29., 19., cv.ink());
                }
                if cv.button(Rect::new(40., 817., 130., 45.), "上一页", page > 0, false) {
                    result = Intent::Page(-1)
                }
                if cv.button(
                    Rect::new(184., 817., 130., 45.),
                    "下一页",
                    page + 1 < pages.len(),
                    false,
                ) {
                    result = Intent::Page(1)
                }
            }
        }
        _ => {
            let help="每次行动：先选行动卡，查看消耗和判定，再选择是否投入一张支援手牌，最后确认。执行期间会播放过场，掷骰与资源变化都写入回执。只有确认后才推进回合。\n\n移动：点地图上的节点，选择循路、慢行或信物提供的特殊移动。每个楼层单独显示；一个坐标可能有多个独立房间，点击圆点可展开。M 回到当前地点所在地图。里世界有 16 区、1101 房间：成功「逐处查看」核实本区通路；跨区、每 24 个里世界回合、死亡或重新进入时，道路会重组。各区有休整处和镜面。\n\n背包：B 打开。消耗品主动使用，器物和信物需要持握，材料用于配方，门卡与结局物品只需携带。选用途前先看数值说明。\n\n成长：C 打开人物卡。新地点、首次线索和信物带来阅历；获得成长点后，可提升一项属性。死亡保留剧情、信物和成长，普通物品按邮筒清单恢复。\n\n阅读：I 或 J 打开档案；V 重看最近行动回执。原文长页支持逐页阅读，翻页不推进回合。\n\n快捷键：1–5 查看当前页的行动牌；Enter/空格确认已选行动，或关闭结果回执；Esc 返回/暂停。自动存档在每次结算后写入，关闭程序后可继续。\n\n完成本章：收集八件信物，在传达室交付，再到校门离校。没有启用结局道具时，原案的结局是「循环」。想继续查明真相，可以先不离校。";
            cv.wrap(help, Rect::new(54., 173., 1290., 580.), 21., cv.ink(), 19);
            if cv.button(
                Rect::new(1050., 815., 345., 45.),
                "分页阅读完整操作说明",
                true,
                true,
            ) {
                result = Intent::Read("操作说明".into(), help.into());
            }
        }
    }
    footer(cv, "档案阅读不推进回合；未取得的原稿成就不会自动解锁。");
    result
}
pub fn achievement_supported(name: &str) -> bool {
    const DATA: &str = include_str!("../data/achievement_support.json");
    let names: Vec<String> = serde_json::from_str(DATA).unwrap_or_default();
    names.iter().any(|n| n == name)
}
fn pause(app: &App, cv: &Canvas) -> Intent {
    cv.text("休息片刻", 420., 144., 40., cv.ink());
    let mut result = Intent::None;
    for (j, (label, action)) in [
        ("继续当前循环".into(), Intent::Back),
        (
            format!(
                "环境音：{}",
                if app.archive.sound {
                    "开启"
                } else {
                    "关闭"
                }
            ),
            Intent::Sound,
        ),
        (
            format!(
                "减少动态效果：{}",
                if app.archive.reduced_motion {
                    "开启"
                } else {
                    "关闭"
                }
            ),
            Intent::Motion,
        ),
        (
            format!("画面亮度 Gamma {:.2}", app.archive.gamma()),
            Intent::Brightness,
        ),
        ("返回标题".into(), Intent::Screen(Screen::Title)),
        ("保存并退出".into(), Intent::Quit),
    ]
    .into_iter()
    .enumerate()
    {
        if cv.button(
            Rect::new(420., 196. + j as f32 * 87., 600., 62.),
            &label,
            true,
            j == 0,
        ) {
            result = action
        }
    }
    cv.wrap(
        "暂停、阅读、分配角色初始点数都不会让走廊中的实体行动。",
        Rect::new(420., 745., 600., 80.),
        19.,
        cv.muted(),
        3,
    );
    result
}
fn ending(app: &App, cv: &Canvas) -> Intent {
    let Some(g) = &app.game else {
        return Intent::Back;
    };
    let ending = match g.status {
        Status::Ended(e) => e,
        _ => return Intent::Back,
    };
    app.visuals.campus(g.world, app.archive.gamma());
    draw_rectangle(0., 0., 1440., 900., Color::new(0.02, 0.04, 0.06, 0.8));
    cv.text("离校记录", 184., 155., 20., c(162, 202, 207));
    cv.text(ending.name(), 180., 260., 65., WHITE);
    cv.wrap(
        ending.text(),
        Rect::new(184., 311., 1050., 250.),
        27.,
        c(226, 231, 224),
        6,
    );
    cv.text(
        &format!(
            "第 {} 回合 · 等级 {} · 信物 {}/8 · 生魂之泪 {}",
            g.turn,
            g.character.level,
            g.relic_count(),
            g.count(Item::Tear)
        ),
        184.,
        606.,
        20.,
        c(180, 207, 209),
    );
    let mut result = Intent::None;
    if cv.button(Rect::new(184., 696., 300., 62.), "回顾档案", true, false) {
        result = Intent::Screen(Screen::Journal)
    }
    if cv.button(
        Rect::new(510., 696., 300., 62.),
        "再填一张入校登记",
        true,
        true,
    ) {
        result = Intent::New
    }
    if cv.button(Rect::new(836., 696., 300., 62.), "返回标题", true, false) {
        result = Intent::Screen(Screen::Title)
    }
    result
}
