use crate::game::Game;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, fs, io::Write, path::PathBuf};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archive {
    pub achievements: BTreeSet<String>,
    pub clues: BTreeSet<String>,
    pub runs: u32,
    pub sound: bool,
    pub reduced_motion: bool,
    pub brightness: u8,
    pub completed_seeds: BTreeSet<u64>,
}
impl Default for Archive {
    fn default() -> Self {
        Self {
            achievements: BTreeSet::new(),
            clues: BTreeSet::new(),
            runs: 0,
            sound: true,
            reduced_motion: false,
            brightness: 2,
            completed_seeds: BTreeSet::new(),
        }
    }
}
impl Archive {
    pub fn gamma(&self) -> f32 {
        [1.45, 1.8, 2.2, 2.5][self.brightness.min(3) as usize]
    }
    pub fn update(&mut self, g: &Game) {
        self.achievements.extend(g.achievements.iter().cloned());
        self.clues.extend(g.clues.iter().cloned());
        if !g.playing() {
            self.completed_seeds.insert(g.seed);
        }
    }
}
pub struct Storage {
    pub dir: PathBuf,
}
impl Default for Storage {
    fn default() -> Self {
        Self::new()
    }
}
impl Storage {
    pub fn new() -> Self {
        let base = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            })
            .unwrap_or_else(|| PathBuf::from("."));
        // A new namespace prevents old numeric map IDs being silently reinterpreted.
        Self {
            dir: base.join("HLG-NightSchool-v2"),
        }
    }
    pub fn load_game(&self) -> Result<Option<Game>, String> {
        let mut error: Option<String> = None;
        for name in ["run.json", "run.backup.json"] {
            let path = self.dir.join(name);
            if !path.exists() {
                continue;
            }
            let result = (|| {
                let bytes = fs::read(&path).map_err(|e| e.to_string())?;
                if bytes.len() > 3_000_000 {
                    return Err("存档异常大".into());
                }
                let g: Game = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                g.validate()?;
                Ok(g)
            })();
            match result {
                Ok(g) => return Ok(Some(g)),
                Err(e) => error = Some(e),
            }
        }
        if let Some(e) = error {
            Err(format!("存档读取未成功：{e}。原文件保留，可以开始新循环。"))
        } else {
            Ok(None)
        }
    }
    pub fn load_archive(&self) -> Archive {
        let mut a: Archive = fs::read(self.dir.join("archive.json"))
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        a.brightness = a.brightness.min(3);
        a
    }
    pub fn save(&self, g: &Game, a: &Archive) -> Result<(), String> {
        g.validate()?;
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let path = self.dir.join("run.json");
        if let Ok(b) = fs::read(&path) {
            if serde_json::from_slice::<Game>(&b).is_ok() {
                fs::write(self.dir.join("run.backup.json"), b).map_err(|e| e.to_string())?;
            }
        }
        self.atomic(
            "run.json",
            &serde_json::to_vec_pretty(g).map_err(|e| e.to_string())?,
        )?;
        self.save_archive(a)
    }
    pub fn save_archive(&self, a: &Archive) -> Result<(), String> {
        fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        self.atomic(
            "archive.json",
            &serde_json::to_vec_pretty(a).map_err(|e| e.to_string())?,
        )
    }
    fn atomic(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        let tmp = self.dir.join(format!("{name}.tmp"));
        let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        f.write_all(bytes).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?;
        drop(f);
        fs::rename(tmp, self.dir.join(name)).map_err(|e| e.to_string())
    }
}
