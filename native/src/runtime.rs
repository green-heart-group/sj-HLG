#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod art;
mod pixel;
#[cfg(feature = "qa")]
#[path = "native_qa.rs"]
mod qa;
#[path = "frontend.rs"]
mod ui;
use hlg_core::{cards::*, character::*, content::*, feedback::Receipt, game::*, save::*};
use macroquad::prelude::*;
use ui::{Intent, Screen};

fn window_conf() -> Conf {
    Conf {
        window_title: "HLG · 理工恐怖故事".into(),
        window_width: 1440,
        window_height: 900,
        high_dpi: true,
        window_resizable: true,
        sample_count: 1,
        platform: macroquad::window::miniquad::conf::Platform {
            swap_interval: Some(1),
            ..Default::default()
        },
        ..Default::default()
    }
}
pub struct Animation {
    pub started: f64,
    pub action: Action,
    pub tactic: Option<Tactic>,
    pub before: Game,
    pub resolved: bool,
    pub label: String,
    pub motion: String,
}
pub struct App {
    pub game: Option<Game>,
    pub archive: Archive,
    pub store: Storage,
    pub screen: Screen,
    pub back: Screen,
    pub floor: i32,
    pub selected: Option<usize>,
    pub node_group: Vec<usize>,
    pub tab: CardTab,
    pub card_page: usize,
    pub modal: Option<Card>,
    pub tactic: Option<Tactic>,
    pub animation: Option<Animation>,
    pub receipt: Option<Receipt>,
    pub receipt_open: bool,
    pub receipt_page: usize,
    pub inv_page: usize,
    pub item: Option<Item>,
    pub journal_tab: usize,
    pub journal_page: usize,
    pub reader_title: String,
    pub reader_text: String,
    pub reader_page: usize,
    pub draft: [u8; 4],
    pub easy: bool,
    pub confirm_restart: bool,
    pub clock: f32,
    pub quit: bool,
    pub toast: String,
    pub toast_until: f64,
    pub save_error: bool,
    pub visuals: pixel::Assets,
}
pub fn arg(key: &str) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    a.iter()
        .position(|s| s == key)
        .and_then(|i| a.get(i + 1))
        .cloned()
}
impl App {
    fn load() -> Self {
        let diagnostic = arg("--diagnose").or_else(|| arg("--qa-output"));
        let store = diagnostic.as_ref().map_or_else(Storage::new, |p| Storage {
            dir: std::path::PathBuf::from(p).join("isolated-save-v2"),
        });
        let mut archive = store.load_archive();
        if diagnostic.is_some() {
            archive.sound = false
        }
        let (game, error) = match store.load_game() {
            Ok(g) => (g, String::new()),
            Err(e) => (None, e),
        };
        Self {
            game,
            archive,
            store,
            screen: Screen::Title,
            back: Screen::Title,
            floor: 1,
            selected: None,
            node_group: vec![],
            tab: CardTab::Explore,
            card_page: 0,
            modal: None,
            tactic: None,
            animation: None,
            receipt: None,
            receipt_open: false,
            receipt_page: 0,
            inv_page: 0,
            item: None,
            journal_tab: 0,
            journal_page: 0,
            reader_title: String::new(),
            reader_text: String::new(),
            reader_page: 0,
            draft: [3, 3, 3, 3],
            easy: false,
            confirm_restart: false,
            clock: 0.,
            quit: false,
            toast: error,
            toast_until: 15.,
            save_error: false,
            visuals: pixel::Assets::new(),
        }
    }
    pub fn visible_cards(&self) -> Vec<Card> {
        self.game
            .as_ref()
            .map(|g| g.cards(self.tab, self.selected))
            .unwrap_or_default()
    }
    pub fn toast(&mut self, msg: impl Into<String>) {
        self.toast = msg.into();
        self.toast_until = get_time() + 5.;
    }
    pub fn save(&mut self) {
        let result = if let Some(g) = &self.game {
            self.archive.update(g);
            self.store.save(g, &self.archive)
        } else {
            self.store.save_archive(&self.archive)
        };
        self.save_error = result.is_err();
        if let Err(e) = result {
            self.toast(format!("保存未成功：{e}"))
        }
    }
    pub fn start(&mut self, seed: u64) {
        match Character::new(self.draft) {
            Ok(ch) => {
                self.game = Some(Game::with_character(seed, self.easy, ch));
                self.screen = Screen::Game;
                self.floor = 1;
                self.receipt = self.game.as_ref().map(Receipt::arrival);
                self.receipt_open = true;
                self.receipt_page = 0;
                self.archive.runs += 1;
                self.selected = None;
                self.card_page = 0;
                self.tab = CardTab::Explore;
                self.modal = None;
                self.animation = None;
                self.confirm_restart = false;
                self.save();
            }
            Err(e) => self.toast(e),
        }
    }
    pub fn open_card(&mut self, card: Card) {
        self.tactic = None;
        self.modal = Some(card);
    }
    pub fn confirm_card(&mut self) {
        let Some(card) = self.modal.clone() else {
            return;
        };
        let Some(g) = self.game.as_ref() else { return };
        if !g.legal_card(&card.action).is_some_and(|c| c.enabled) {
            self.toast("行动条件已变化，请重新查看牌面。");
            return;
        }
        let motion = match card.action {
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
        };
        self.animation = Some(Animation {
            started: get_time(),
            action: card.action,
            tactic: self.tactic,
            before: g.clone(),
            resolved: false,
            label: card.title,
            motion: motion.into(),
        });
        self.modal = None;
        self.receipt_open = false;
        self.node_group.clear();
        self.screen = Screen::Game;
    }
    pub fn update(&mut self) {
        let elapsed = self.animation.as_ref().map(|a| get_time() - a.started);
        if elapsed.is_some_and(|t| t >= 0.65)
            && self.animation.as_ref().is_some_and(|a| !a.resolved)
        {
            let a = self.animation.as_mut().unwrap();
            a.resolved = true;
            if let Some(g) = self.game.as_mut() {
                g.play(a.action.clone(), a.tactic);
                self.receipt = Some(Receipt::capture(&a.before, g));
                self.floor = g.map_index();
                self.selected = None;
                self.card_page = 0;
                self.tab = if g.in_combat() {
                    CardTab::Combat
                } else {
                    CardTab::Explore
                };
            }
            self.save();
        }
        if elapsed.is_some_and(|t| t >= 1.65) {
            self.animation = None;
            self.receipt_open = self.receipt.as_ref().is_some_and(|r| r.major);
            self.receipt_page = 0;
            if self.game.as_ref().is_some_and(|g| !g.playing()) {
                self.screen = Screen::End;
                self.receipt_open = false;
            }
        }
    }
    pub fn intent(&mut self, i: Intent) {
        if self.animation.is_some() {
            return;
        }
        match i {
            Intent::None => {}
            Intent::New => {
                if self
                    .game
                    .as_ref()
                    .is_some_and(|g| g.playing() && g.turn > 0)
                    && !self.confirm_restart
                {
                    self.confirm_restart = true;
                    self.toast("当前循环仍在进行。再点一次开始新循环，会替换本局；档案保留。");
                } else {
                    self.screen = Screen::Create;
                    self.draft = [3, 3, 3, 3];
                }
            }
            Intent::Begin => {
                let seed = arg("--seed")
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_nanos() as u64
                            % 999_983
                            + 1
                    });
                self.start(seed);
            }
            Intent::Continue => {
                if let Some(g) = &self.game {
                    self.floor = g.map_index();
                    self.receipt = Some(Receipt::arrival(g));
                    self.screen = if g.playing() {
                        Screen::Game
                    } else {
                        Screen::End
                    };
                }
            }
            Intent::Achievements => {
                self.back = self.screen;
                self.screen = Screen::Journal;
                self.journal_tab = 1;
                self.journal_page = 0;
            }
            Intent::Screen(s) => {
                self.back = self.screen;
                self.screen = s;
                self.modal = None;
                self.node_group.clear();
            }
            Intent::Back => {
                if self.modal.is_some() {
                    self.modal = None;
                    self.tactic = None;
                } else if self.receipt_open {
                    self.receipt_open = false;
                } else if !self.node_group.is_empty() {
                    self.node_group.clear();
                } else {
                    self.screen = if matches!(self.screen, Screen::Reader) {
                        self.back
                    } else if self.game.as_ref().is_some_and(|g| !g.playing()) {
                        Screen::End
                    } else if self.game.is_some() {
                        Screen::Game
                    } else {
                        Screen::Title
                    };
                }
            }
            Intent::Floor(f) => self.floor = f,
            Intent::Select(n) => {
                self.selected = Some(n);
                self.node_group.clear();
                self.tab = CardTab::Move;
                self.card_page = 0;
            }
            Intent::Group(group) => self.node_group = group,
            Intent::Tab(t) => {
                self.tab = t;
                self.card_page = 0;
            }
            Intent::Card(index) => {
                if let Some(c) = self.visible_cards().get(index) {
                    self.open_card(c.clone());
                }
            }
            Intent::Open(card) => self.open_card(card),
            Intent::Confirm => self.confirm_card(),
            Intent::Tactic(t) => {
                self.tactic = if self.tactic == Some(t) {
                    None
                } else {
                    Some(t)
                }
            }
            Intent::CardPage(d) => self.card_page = (self.card_page as i32 + d).max(0) as usize,
            Intent::InvPage(d) => self.inv_page = (self.inv_page as i32 + d).max(0) as usize,
            Intent::Item(i) => {
                self.item = Some(i);
                self.card_page = 0;
            }
            Intent::JournalTab(t) => {
                self.journal_tab = t;
                self.journal_page = 0;
            }
            Intent::Page(d) => {
                if self.screen == Screen::Reader {
                    self.reader_page = (self.reader_page as i32 + d).max(0) as usize;
                } else if self.receipt_open {
                    self.receipt_page = (self.receipt_page as i32 + d).max(0) as usize;
                } else {
                    self.journal_page = (self.journal_page as i32 + d).max(0) as usize;
                }
            }
            Intent::Read(title, text) => {
                self.back = self.screen;
                self.reader_title = title;
                self.reader_text = text;
                self.reader_page = 0;
                self.screen = Screen::Reader;
            }
            Intent::Receipt => {
                self.receipt_open = true;
                self.receipt_page = 0;
            }
            Intent::CloseReceipt => self.receipt_open = false,
            Intent::Draft(i, d) => {
                let v = self.draft[i] as i32 + d;
                if (1..=5).contains(&v)
                    && (d < 0 || self.draft.iter().map(|&x| x as u16).sum::<u16>() < 12)
                {
                    self.draft[i] = v as u8
                }
            }
            Intent::Preset(v) => self.draft = v,
            Intent::Mode => self.easy = !self.easy,
            Intent::Sound => {
                self.archive.sound = !self.archive.sound;
                self.save()
            }
            Intent::Motion => {
                self.archive.reduced_motion = !self.archive.reduced_motion;
                self.save()
            }
            Intent::Brightness => {
                self.archive.brightness = (self.archive.brightness + 1) % 4;
                self.save()
            }
            Intent::Quit => {
                self.save();
                self.quit = true
            }
        }
    }
    fn keyboard(&self) -> Intent {
        if self.animation.is_some() {
            return Intent::None;
        }
        if is_key_pressed(KeyCode::Escape) {
            return if self.screen == Screen::Game
                && self.modal.is_none()
                && !self.receipt_open
                && self.node_group.is_empty()
            {
                Intent::Screen(Screen::Pause)
            } else {
                Intent::Back
            };
        }
        if self.modal.is_some() {
            return if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                Intent::Confirm
            } else {
                Intent::None
            };
        }
        if self.receipt_open {
            return if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                Intent::CloseReceipt
            } else {
                Intent::None
            };
        }
        if self.screen == Screen::Game {
            if is_key_pressed(KeyCode::B) {
                return Intent::Screen(Screen::Inventory);
            }
            if is_key_pressed(KeyCode::I) || is_key_pressed(KeyCode::J) {
                return Intent::Screen(Screen::Journal);
            }
            if is_key_pressed(KeyCode::A) {
                return Intent::Achievements;
            }
            if is_key_pressed(KeyCode::C) {
                return Intent::Screen(Screen::Character);
            }
            if is_key_pressed(KeyCode::V) {
                return Intent::Receipt;
            }
            if is_key_pressed(KeyCode::M) {
                return self
                    .game
                    .as_ref()
                    .map_or(Intent::None, |g| Intent::Floor(g.map_index()));
            }
            for (i, k) in [
                KeyCode::Key1,
                KeyCode::Key2,
                KeyCode::Key3,
                KeyCode::Key4,
                KeyCode::Key5,
            ]
            .into_iter()
            .enumerate()
            {
                if is_key_pressed(k) {
                    return Intent::Card(self.card_page * 5 + i);
                }
            }
        }
        Intent::None
    }
}
struct Ambience {
    last: Option<(bool, World)>,
}
impl Ambience {
    fn sync(&mut self, on: bool, w: World) {
        if self.last == Some((on, w)) {
            return;
        }
        self.last = Some((on, w));
        #[cfg(windows)]
        unsafe {
            #[link(name = "winmm")]
            extern "system" {
                fn PlaySoundW(sound: *const u16, module: isize, flags: u32) -> i32;
            }
            if on {
                let bytes: &'static [u8] = if w == World::Surface {
                    include_bytes!("../assets/surface.wav")
                } else {
                    include_bytes!("../assets/echo.wav")
                };
                PlaySoundW(
                    bytes.as_ptr() as *const u16,
                    0,
                    0x0001 | 0x0004 | 0x0008 | 0x0002,
                );
            } else {
                PlaySoundW(std::ptr::null(), 0, 0);
            }
        }
    }
}
#[macroquad::main(window_conf)]
async fn main() {
    let font = load_ttf_font_from_bytes(include_bytes!("../assets/NotoSansSC.ttf"))
        .expect("embedded font");
    let target = render_target(1440, 900);
    target.texture.set_filter(FilterMode::Linear);
    let mut camera = Camera2D::from_display_rect(Rect::new(0., 0., 1440., 900.));
    camera.render_target = Some(target.clone());
    let mut app = App::load();
    let mut ambience = Ambience { last: None };
    let mut frames = 0;
    #[cfg(feature = "qa")]
    let mut harness = qa::Harness::new(&mut app);
    loop {
        if app.quit {
            break;
        }
        app.clock = if app.archive.reduced_motion {
            0.
        } else {
            get_time() as f32
        };
        app.update();
        let scale = (screen_width() / 1440.).min(screen_height() / 900.);
        let offset = vec2(
            (screen_width() - 1440. * scale) / 2.,
            (screen_height() - 900. * scale) / 2.,
        );
        let (mx, my) = mouse_position();
        let mouse = (vec2(mx, my) - offset) / scale;
        #[allow(unused_mut)]
        let mut mouse = mouse;
        #[allow(unused_mut)]
        let mut key = app.keyboard();
        #[allow(unused_mut)]
        let mut click = is_mouse_button_pressed(MouseButton::Left);
        #[cfg(feature = "qa")]
        if let Some(h) = &harness {
            let p = h.input();
            mouse = p.0;
            click = p.1;
            key = Intent::None;
        }
        set_camera(&camera);
        clear_background(BLACK);
        let mut cv = ui::Canvas::new(&font, mouse, click && matches!(key, Intent::None));
        let action = ui::draw(&app, &mut cv);
        if get_time() < app.toast_until && !app.toast.is_empty() {
            cv.toast(&app.toast)
        }
        set_default_camera();
        clear_background(BLACK);
        draw_texture_ex(
            &target.texture,
            offset.x,
            offset.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(1440. * scale, 900. * scale)),
                flip_y: true,
                ..Default::default()
            },
        );
        app.intent(if matches!(key, Intent::None) {
            action
        } else {
            key
        });
        let w = app.game.as_ref().map_or(World::Surface, |g| g.world);
        ambience.sync(app.archive.sound, w);
        frames += 1;
        if frames == 30 {
            if let Some(p) = arg("--diagnose") {
                let dir = std::path::PathBuf::from(p);
                std::fs::create_dir_all(&dir).unwrap();
                get_screen_data().export_png(dir.join("startup.png").to_str().unwrap());
                std::fs::write(dir.join("diagnostics.json"),serde_json::to_vec_pretty(&serde_json::json!({"result":"PASS","version":env!("CARGO_PKG_VERSION"),"build_fingerprint":env!("HLG_BUILD_FINGERPRINT"),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"gamma":app.archive.gamma(),"qa_feature":cfg!(feature="qa"),"frames_rendered":frames,"renderer":"native miniquad"})).unwrap()).unwrap();
                app.quit = true;
            }
        }
        #[cfg(feature = "qa")]
        if let Some(h) = &mut harness {
            h.after_frame(&mut app);
        }
        next_frame().await;
    }
    ambience.sync(false, World::Surface);
}
