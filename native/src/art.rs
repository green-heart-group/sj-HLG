use hlg_core::content::World;
use macroquad::prelude::*;
pub fn c(r: u8, g: u8, b: u8) -> Color {
    Color::from_rgba(r, g, b, 255)
}
pub fn alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}
pub const POST_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position=Projection*Model*vec4(position,1.0);
    uv=texcoord;
}
"#;
pub const POST_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
uniform sampler2D Texture;
uniform float Gamma;
uniform vec2 PixelGrid;
void main() {
    vec2 sampleUv=clamp(uv,vec2(0.5)/PixelGrid,vec2(1.0)-vec2(0.5)/PixelGrid);
    vec3 rgb=texture2D(Texture,sampleUv).rgb;
    gl_FragColor=vec4(pow(max(rgb,vec3(0.0)),vec3(1.0/max(Gamma,1.0))),1.0);
}
"#;
pub fn meter(r: Rect, fraction: f32, col: Color, world: World, sanity: i32, time: f32) {
    let fraction = fraction.clamp(0., 1.);
    if world == World::Surface {
        draw_rectangle(r.x, r.y, r.w, r.h, c(159, 177, 149));
        draw_rectangle(r.x, r.y, r.w * fraction, r.h, col);
    } else {
        let strength = 0.8 + (100 - sanity).max(0) as f32 / 32.;
        let count = (r.w / 5.).ceil() as usize;
        for i in 0..count {
            let xx = i as f32 * 5.;
            let width = 5f32.min(r.w - xx);
            let yy = r.y + ((i as f32 * 0.65 + time * 0.5).sin() * strength).round();
            draw_rectangle(r.x + xx, yy, width, r.h, c(55, 34, 59));
            let fill = (r.w * fraction - xx).clamp(0., width);
            draw_rectangle(r.x + xx, yy, fill, r.h, col);
            if (i + (time * 0.3) as usize) % 17 == 0 {
                draw_rectangle(r.x + xx, yy + 1., fill, 1., c(38, 23, 46));
            }
        }
    }
}
