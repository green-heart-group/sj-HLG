//! Native in-app input and framebuffer checks. Never controls the OS or another app.
use crate::{
    arg, ui,
    ui::{Intent, Screen},
    App,
};
use hlg_core::{cards::*, content::*, feedback::Receipt, game::*, save::Archive};
use macroquad::prelude::*;
use std::{collections::BTreeSet, path::PathBuf};
pub struct Harness {
    phase: u32,
    frame: u32,
    total: u32,
    out: PathBuf,
    states: PathBuf,
    checks: Vec<String>,
    captures: BTreeSet<String>,
    started: f64,
    initial_hand: Vec<Tactic>,
    echo_revision: u32,
    echo_room: usize,
}
impl Harness {
    pub fn new(app: &mut App) -> Option<Self> {
        let out = PathBuf::from(arg("--qa-output")?);
        std::fs::create_dir_all(&out).unwrap();
        app.store.dir = out.join("isolated-save-v2");
        app.archive = Archive::default();
        app.archive.sound = false;
        app.game = None;
        app.screen = Screen::Title;
        app.toast.clear();
        app.toast_until = 0.;
        app.modal = None;
        app.receipt_open = false;
        let states = out.parent().unwrap().parent().unwrap().join("qa-states");
        Some(Self {
            phase: 0,
            frame: 0,
            total: 0,
            out,
            states,
            checks: vec![],
            captures: BTreeSet::new(),
            started: get_time(),
            initial_hand: vec![],
            echo_revision: 0,
            echo_room: 0,
        })
    }
    pub fn input(&self) -> (Vec2, bool) {
        let p = match self.phase {
            1 => vec2(274., 543.),
            2 => vec2(666., 751.),
            3 => vec2(1215., 759.),
            4 => vec2(1057., 786.),
            5 => ui::map_pos(node("reception")),
            6 => vec2(153., 713.),
            7 => vec2(355., 650.),
            8 => vec2(1045., 775.),
            9 => vec2(90., 645.),
            10 => vec2(151., 714.),
            11 => vec2(1045., 775.),
            12 => vec2(1027., 47.),
            13 => vec2(115., 201.),
            14 => vec2(1326., 50.),
            16 => vec2(719., 716.),
            17 => vec2(1045., 775.),
            19 => vec2(1135., 48.),
            20 => vec2(320., 120.),
            21 => vec2(1294., 312.),
            22 => vec2(1260., 845.),
            29 => vec2(940., 132.),
            30 => vec2(930., 215.),
            32 => vec2(1045., 775.),
            33 => vec2(1057., 786.),
            _ => vec2(-100., -100.),
        };
        (
            p,
            self.frame == 2 && matches!(self.phase,1..=14|16|17|19..=22|29|30|32|33),
        )
    }
    fn check(&mut self, name: &str, ok: bool) {
        assert!(ok, "native check failed phase {}: {name}", self.phase);
        self.checks.push(name.into());
    }
    fn capture(&mut self, name: &str) {
        if self.captures.insert(name.into()) {
            get_screen_data().export_png(self.out.join(format!("v2-{name}.png")).to_str().unwrap());
        }
    }
    fn advance(&mut self) {
        self.phase += 1;
        self.frame = 0;
    }
    fn load(&mut self, app: &mut App, name: &str) {
        let bytes = std::fs::read(self.states.join(format!("v2-{name}.json"))).unwrap();
        let g: Game = serde_json::from_slice(&bytes).unwrap();
        g.validate().unwrap();
        app.archive.update(&g);
        app.floor = g.map_index();
        app.receipt = Some(Receipt::arrival(&g));
        app.game = Some(g);
        app.screen = Screen::Game;
        app.modal = None;
        app.animation = None;
        app.receipt_open = false;
        app.selected = None;
        app.tab = CardTab::Explore;
        app.card_page = 0;
        app.toast.clear();
    }
    pub fn after_frame(&mut self, app: &mut App) {
        self.frame += 1;
        self.total += 1;
        assert!(get_time() - self.started < 90., "native QA timeout");
        if self.phase == 29 && self.frame == 1 {
            self.capture("27-full-echo-zone");
        }
        if self.frame < 7 {
            return;
        }
        match self.phase {
            0 => {
                self.capture("00-title");
                self.advance();
            }
            1 => {
                self.check(
                    "new_game_opens_character_creation",
                    app.screen == Screen::Create,
                );
                self.capture("01-create");
                self.advance();
            }
            2 => {
                self.check(
                    "character_preset_changes_four_attributes",
                    app.draft == [2, 3, 5, 2],
                );
                self.advance();
            }
            3 => {
                self.check(
                    "created_character_is_used",
                    app.game.as_ref().unwrap().character.values == [2, 3, 5, 2],
                );
                self.check("arrival_receipt_is_visible", app.receipt_open);
                self.capture("02-arrival");
                self.advance();
            }
            4 => {
                self.check(
                    "acknowledge_does_not_advance_turn",
                    app.game.as_ref().unwrap().turn == 0 && !app.receipt_open,
                );
                self.capture("03-surface");
                self.advance();
            }
            5 => {
                self.check(
                    "map_click_selects_source_coordinate",
                    app.selected == Some(node("reception")) && app.tab == CardTab::Move,
                );
                self.advance();
            }
            6 => {
                self.check(
                    "action_click_opens_preview_without_spending",
                    app.modal.is_some() && app.game.as_ref().unwrap().turn == 0,
                );
                self.capture("04-action-preview");
                self.initial_hand = app.game.as_ref().unwrap().hand.clone();
                self.advance();
            }
            7 => {
                self.check(
                    "support_card_is_selected_before_resolution",
                    app.tactic.is_some(),
                );
                self.capture("05-support-selected");
                self.advance();
            }
            8 => {
                if let Some(a) = &app.animation {
                    let p = get_time() - a.started;
                    if p > 0.3 && p < 0.6 {
                        self.capture("06-walking-animation");
                    }
                    if p > 1.2 {
                        self.capture("07-roll-animation");
                    }
                } else {
                    let g = app.game.as_ref().unwrap();
                    self.check(
                        "movement_reaches_selected_coordinate",
                        g.current == node("reception"),
                    );
                    self.check("one_action_advances_exactly_one_turn", g.turn == 1);
                    self.check(
                        "support_discard_and_replenish",
                        g.hand.len() == 3 && g.discard_pile.len() == 1,
                    );
                    self.capture("08-movement-result");
                    self.advance();
                }
            }
            9 => {
                self.check(
                    "exploration_tab_contains_context_card",
                    app.visible_cards()
                        .iter()
                        .any(|c| c.action == Action::Quest("guide".into())),
                );
                self.advance();
            }
            10 => {
                self.check(
                    "context_card_preview",
                    app.modal
                        .as_ref()
                        .is_some_and(|c| c.action == Action::Quest("guide".into())),
                );
                self.advance();
            }
            11 => {
                if app.animation.is_none() {
                    self.check(
                        "context_event_sets_actual_progress",
                        app.game.as_ref().unwrap().flag("guide"),
                    );
                    self.capture("09-event-result");
                    self.advance();
                }
            }
            12 => {
                self.check(
                    "inventory_button_opens_inventory",
                    app.screen == Screen::Inventory,
                );
                self.capture("10-inventory");
                self.advance();
            }
            13 => {
                self.check(
                    "item_selection_does_not_use_item",
                    app.item == Some(Item::Water)
                        && app.game.as_ref().unwrap().count(Item::Water) == 1,
                );
                self.capture("11-item-details");
                self.advance();
            }
            14 => {
                self.check("inventory_returns_to_game", app.screen == Screen::Game);
                self.load(app, "combat");
                app.tab = CardTab::Combat;
                self.advance();
            }
            15 => {
                self.check(
                    "combat_snapshot_has_enemy_intent",
                    app.game.as_ref().unwrap().in_combat(),
                );
                self.capture("12-combat");
                self.advance();
            }
            16 => {
                self.check(
                    "lamp_combat_card_preview",
                    app.modal
                        .as_ref()
                        .is_some_and(|c| c.action == Action::Attack(AttackMode::Lamp)),
                );
                self.capture("13-lamp-preview");
                self.advance();
            }
            17 => {
                if let Some(a) = &app.animation {
                    let p = get_time() - a.started;
                    if p > 0.45 && p < 0.62 {
                        self.capture("14-combat-animation");
                    }
                    if p > 1.2 {
                        self.capture("15-combat-dice");
                    }
                } else {
                    self.check(
                        "lamp_actually_repels_wxh",
                        !app.game.as_ref().unwrap().in_combat(),
                    );
                    self.capture("16-combat-result");
                    self.load(app, "complete");
                    app.screen = Screen::Inventory;
                    app.item = Some(Item::Wenjin);
                    self.advance();
                }
            }
            18 => {
                self.capture("17-full-inventory");
                app.screen = Screen::Game;
                self.advance();
            }
            19 => {
                self.check("journal_button", app.screen == Screen::Journal);
                self.capture("18-documents");
                self.advance();
            }
            20 => {
                self.check("achievement_catalog_tab", app.journal_tab == 1);
                self.capture("19-achievements");
                self.advance();
            }
            21 => {
                self.check(
                    "achievement_original_text_reader",
                    app.screen == Screen::Reader
                        && app.reader_text.contains("你设法取得了一张门卡，"),
                );
                self.capture("20-original-achievement");
                self.advance();
            }
            22 => {
                self.check("reader_returns_to_journal", app.screen == Screen::Journal);
                let d = DOCUMENTS.iter().find(|d| d.id == "diary1").unwrap();
                app.intent(Intent::Read(d.title.clone(), d.text.clone()));
                self.advance();
            }
            23 => {
                self.capture("21-long-document");
                app.intent(Intent::Page(1));
                self.advance();
            }
            24 => {
                self.capture("22-document-next-page");
                self.load(app, "echo");
                self.advance();
            }
            25 => {
                self.capture("23-echo");
                app.screen = Screen::Character;
                self.advance();
            }
            26 => {
                self.capture("24-growth");
                self.load(app, "ending");
                app.screen = Screen::End;
                self.advance();
            }
            27 => {
                self.capture("25-ending");
                app.screen = Screen::Pause;
                self.advance();
            }
            28 => {
                self.capture("26-settings");
                self.check(
                    "default_gamma_2_2",
                    (app.archive.gamma() - 2.2).abs() < 0.01,
                );
                app.save();
                self.check(
                    "v2_save_round_trip",
                    app.store.load_game().unwrap().as_ref() == app.game.as_ref(),
                );
                self.load(app, "rogue");
                self.advance();
            }
            29 => {
                self.check("echo_overview_opens", app.floor == -1);
                self.capture("27-echo-overview");
                self.advance();
            }
            30 => {
                self.check("echo_zone_can_be_selected", app.floor == 0);
                self.capture("28-echo-zone");
                let g = app.game.as_ref().unwrap();
                app.floor = g.map_index();
                self.echo_revision = g.echo.revision;
                self.echo_room = g.current;
                app.open_card(g.legal_card(&Action::Reshape).unwrap());
                self.advance();
            }
            31 => {
                self.check(
                    "reshape_preview_does_not_change_map",
                    app.game.as_ref().unwrap().echo.revision == self.echo_revision,
                );
                self.capture("29-reshape-preview");
                self.advance();
            }
            32 => {
                if let Some(a) = &app.animation {
                    if get_time() - a.started > 0.8 {
                        self.capture("30-rearrangement-animation");
                    }
                } else {
                    let g = app.game.as_ref().unwrap();
                    self.check(
                        "reshape_changes_actual_graph",
                        g.echo.revision > self.echo_revision,
                    );
                    self.check("reshape_keeps_current_room", g.current == self.echo_room);
                    self.check(
                        "reshape_receipt_names_changed_routes",
                        g.last.notes.iter().any(|x| x.contains("旧通路")),
                    );
                    self.capture("31-rearrangement-receipt");
                    self.advance();
                }
            }
            33 => {
                self.check("reshape_receipt_can_close", !app.receipt_open);
                self.capture("32-rearranged-map");
                self.advance();
            }
            34 => {
                app.save();
                self.check(
                    "procedural_save_round_trip",
                    app.store.load_game().unwrap().as_ref() == app.game.as_ref(),
                );
                std::fs::write(self.out.join("native-verification-v2.json"),serde_json::to_vec_pretty(&serde_json::json!({
                    "result":"PASS","version":env!("CARGO_PKG_VERSION"),"build_fingerprint":env!("HLG_BUILD_FINGERPRINT"),"qa_feature":true,"checks":self.checks,
                    "screenshots":self.captures,"frames":self.total,"duration_seconds":get_time()-self.started,
                    "input":"in-app Canvas clicks; no OS automation","snapshot_source":"public-action engine routes for combat, completion, echo and ending fixtures"
                })).unwrap()).unwrap();
                app.quit = true;
                self.advance();
            }
            _ => {}
        }
    }
}
