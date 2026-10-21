//! Embedded contemporary-school atlases and animated school-uniform portraits.
use hlg_core::content::World;
use macroquad::prelude::*;

pub struct Assets {
    faces: Texture2D,
    atlases: [Texture2D; 2],
    gamma: Material,
    items: [Texture2D; 2],
    student: Texture2D,
    characters: Texture2D,
    bounds: std::collections::BTreeMap<String, Vec<[f32; 4]>>,
}
impl Assets {
    pub fn new() -> Self {
        let faces = Texture2D::from_file_with_format(
            include_bytes!("../assets/player-portraits.png"),
            Some(ImageFormat::Png),
        );
        faces.set_filter(FilterMode::Nearest);
        let atlas = Texture2D::from_file_with_format(
            include_bytes!("../assets/school-scenes.png"),
            Some(ImageFormat::Png),
        );
        atlas.set_filter(FilterMode::Linear);
        let details = Texture2D::from_file_with_format(
            include_bytes!("../assets/school-details.png"),
            Some(ImageFormat::Png),
        );
        details.set_filter(FilterMode::Linear);
        let gamma = load_material(
            ShaderSource::Glsl {
                vertex: crate::art::POST_VERTEX,
                fragment: crate::art::POST_FRAGMENT,
            },
            MaterialParams {
                uniforms: vec![
                    UniformDesc::new("Gamma", UniformType::Float1),
                    UniformDesc::new("PixelGrid", UniformType::Float2),
                ],
                ..Default::default()
            },
        )
        .expect("image gamma");
        Self {
            faces,
            atlases: [atlas, details],
            gamma,
            items: [
                Texture2D::from_file_with_format(
                    include_bytes!("../assets/items-school.png"),
                    Some(ImageFormat::Png),
                ),
                Texture2D::from_file_with_format(
                    include_bytes!("../assets/items-lab.png"),
                    Some(ImageFormat::Png),
                ),
            ],
            student: Texture2D::from_file_with_format(
                include_bytes!("../assets/student-actions.png"),
                Some(ImageFormat::Png),
            ),
            characters: Texture2D::from_file_with_format(
                include_bytes!("../assets/campus-characters.png"),
                Some(ImageFormat::Png),
            ),
            bounds: serde_json::from_str(include_str!("../data/sprite_bounds.json"))
                .expect("sprite UV bounds"),
        }
    }
    fn sprite(texture: &Texture2D, r: Rect, bounds: [f32; 4], nominal: Option<Vec2>, tint: Color) {
        let [x, y, cw, ch] = bounds;
        texture.set_filter(FilterMode::Linear);
        let scale = nominal.map_or((r.w / cw).min(r.h / ch), |v| (r.w / v.x).min(r.h / v.y));
        let w = cw * scale;
        let h = ch * scale;
        draw_texture_ex(
            texture,
            r.x + (r.w - w) / 2.,
            if nominal.is_some() {
                r.y + r.h - h
            } else {
                r.y + (r.h - h) / 2.
            },
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(w, h)),
                source: Some(Rect::new(x, y, cw, ch)),
                ..Default::default()
            },
        );
    }
    pub fn item(&self, r: Rect, index: usize, tint: Color) {
        let key = if index < 24 {
            "items-school"
        } else {
            "items-lab"
        };
        Self::sprite(
            &self.items[(index / 24).min(1)],
            r,
            self.bounds[key][index % 24],
            None,
            tint,
        );
    }
    pub fn student(&self, r: Rect, index: usize, tint: Color) {
        Self::sprite(
            &self.student,
            r,
            self.bounds["student-actions"][index.min(15)],
            Some(vec2(384., 280.)),
            tint,
        );
    }
    pub fn enemy(&self, r: Rect, kind: hlg_core::game::EnemyKind, tint: Color) {
        use hlg_core::game::EnemyKind::*;
        let i = match kind {
            Wxh => 0,
            Zyy => 1,
            Copy => 2,
            Assimilated => 3,
            Ll => 4,
            Sgf => 5,
            Hjf => 6,
            Ljx => 7,
        };
        Self::sprite(
            &self.characters,
            r,
            self.bounds["campus-characters"][i],
            Some(vec2(384., 512.)),
            tint,
        );
    }
    fn image(&self, r: Rect, sheet: usize, index: usize, brightness: f32) {
        let atlas = &self.atlases[sheet];
        let cw = atlas.width() / 3.;
        let ch = atlas.height() / 4.;
        let mut source = Rect::new(
            (index % 3) as f32 * cw + 2.,
            (index / 3) as f32 * ch + 2.,
            cw - 4.,
            ch - 4.,
        );
        let ratio = r.w / r.h;
        if source.w / source.h > ratio {
            let w = source.h * ratio;
            source.x += (source.w - w) / 2.;
            source.w = w;
        } else {
            let h = source.w / ratio;
            source.y += (source.h - h) / 2.;
            source.h = h;
        }
        // Brightness affects illustrations only; interface ink keeps normal window contrast.
        self.gamma.set_uniform("Gamma", (brightness / 1.65).max(1.));
        let pixel_step = if r.w > 1000. { 1. } else { 2. };
        self.gamma.set_uniform(
            "PixelGrid",
            vec2(atlas.width() / pixel_step, atlas.height() / pixel_step),
        );
        gl_use_material(&self.gamma);
        draw_texture_ex(
            atlas,
            r.x,
            r.y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(r.w, r.h)),
                source: Some(source),
                ..Default::default()
            },
        );
        gl_use_default_material();
    }
    pub fn campus(&self, world: World, brightness: f32) {
        self.image(
            Rect::new(0., 0., 1440., 900.),
            0,
            world.index() * 6,
            brightness,
        );
        for x in 0..1000 {
            let opacity = 0.96 * (1. - x as f32 / 1000.).powi(2);
            draw_rectangle(
                x as f32,
                0.,
                1.,
                900.,
                Color::new(0.025, 0.035, 0.04, opacity),
            );
        }
    }
    pub fn scene(
        &self,
        r: Rect,
        id: usize,
        world: World,
        time: f32,
        danger: bool,
        brightness: f32,
    ) {
        let (sheet, index) = match id {
            6 => (1, 0),
            5 | 16 => (1, 1),
            11 => (1, 2),
            12 => (1, 3),
            7 => (1, 4),
            23 => (1, 5),
            0 | 2 | 13 => (0, 0),
            1 | 3 | 4 => (0, 1),
            17 => (0, 2),
            9 | 10 | 18 | 20 | 21 | 25 => (0, 3),
            8 | 14 | 22 => (0, 4),
            _ => (0, 5),
        };
        self.image(r, sheet, index + world.index() * 6, brightness);
        if world == World::Echo {
            for i in 0..22 {
                let x = r.x + ((i * 137 + (time * 2.) as i32) % r.w as i32) as f32;
                let y = r.y + ((i * 59 + 17) % r.h as i32) as f32;
                draw_rectangle(
                    x,
                    y,
                    if i % 4 == 0 { 7. } else { 2. },
                    2.,
                    if i % 3 == 0 {
                        Color::from_rgba(150, 64, 124, 180)
                    } else {
                        Color::from_rgba(22, 9, 30, 210)
                    },
                );
            }
        }
        if danger {
            let x = r.x + r.w * 0.50;
            let y = r.y + r.h * 0.43;
            let pulse = 0.45 + (time * 1.5).sin() * 0.12;
            draw_rectangle_lines(
                r.x + 3.,
                r.y + 3.,
                r.w - 6.,
                r.h - 6.,
                3.,
                Color::new(0.65, 0.22, 0.25, pulse),
            );
            for dx in [-9., 7.] {
                draw_rectangle(
                    x + dx - 2.,
                    y - 2.,
                    6.,
                    6.,
                    Color::new(0.35, 0.08, 0.12, 0.5),
                );
                draw_rectangle(x + dx, y, 2., 2., Color::new(0.93, 0.61, 0.45, 0.85));
            }
        }
        draw_rectangle_lines(r.x, r.y, r.w, r.h, 2., Color::from_rgba(70, 79, 64, 255));
    }
    pub fn portrait(&self, r: Rect, hp: i32, world: World, time: f32, hurt: bool, motion: bool) {
        let grade = if hp > 70 {
            0
        } else if hp > 35 {
            1
        } else {
            2
        };
        let pose = if hurt {
            4
        } else if !motion {
            1
        } else if time % 5.3 < 0.16 {
            3
        } else {
            (time / 2.8) as usize % 3
        };
        let texture = &self.faces;
        let cw = texture.width() / 5.;
        let ch = texture.height() / 3.;
        let source = Rect::new(
            pose as f32 * cw + 2.,
            grade as f32 * ch + 2.,
            cw - 4.,
            ch - 4.,
        );
        draw_texture_ex(
            texture,
            r.x,
            r.y,
            if world == World::Echo {
                Color::from_rgba(192, 177, 255, 255)
            } else {
                WHITE
            },
            DrawTextureParams {
                dest_size: Some(vec2(r.w, r.h)),
                source: Some(source),
                ..Default::default()
            },
        );
        draw_rectangle_lines(
            r.x - 1.,
            r.y - 1.,
            r.w + 2.,
            r.h + 2.,
            2.,
            if world == World::Surface {
                Color::from_rgba(73, 95, 70, 255)
            } else {
                Color::from_rgba(141, 78, 109, 255)
            },
        );
    }
}
