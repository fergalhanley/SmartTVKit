//! SmartTVKit rendering feasibility demo. All principal UI graphics are drawn in
//! Rust via Macroquad/GL, rather than browser DOM widgets. The scene comes
//! from an A2UI v0.9.1 JSONL message stream.

use macroquad::prelude::*;
use smarttvkit_demo::{Direction, Navigation, Scene, BUNDLED_A2UI};

const VIEW_W: f32 = 1280.0;
const VIEW_H: f32 = 720.0;
const CARD_W: f32 = 230.0;
const CARD_GAP: f32 = 24.0;
const CARD_LEFT: f32 = 108.0;
const ROW_YS: [f32; 2] = [269.0, 509.0];
const CARD_HEIGHT: f32 = 153.0;

#[cfg(target_arch = "wasm32")]
const CONFIG_PATH: &str = "../app/home.a2ui.jsonl";
#[cfg(not(target_arch = "wasm32"))]
const CONFIG_PATH: &str = "app/home.a2ui.jsonl";

/// Set up top-left-origin UI coordinates in a fixed 1280x720 design space.
///
/// Macroquad's screen camera *already* reverses the Y axis internally.
/// Therefore a POSITIVE Y zoom makes increasing UI Y move down the screen.
/// Camera2D::from_display_rect uses negative Y zoom, which mirrors this UI.
fn ui_camera() -> Camera2D {
    Camera2D {
        target: vec2(VIEW_W * 0.5, VIEW_H * 0.5),
        zoom: vec2(2.0 / VIEW_W, 2.0 / VIEW_H),
        ..Default::default()
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "SmartTVKit — Rust UI prototype".to_string(),
        window_width: VIEW_W as i32,
        window_height: VIEW_H as i32,
        window_resizable: true,
        ..Default::default()
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}

fn translucent(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, alpha)
}

async fn load_scene() -> Result<Scene, String> {
    #[cfg(target_arch = "wasm32")]
    let path = format!("{CONFIG_PATH}?t={}", get_time());
    #[cfg(not(target_arch = "wasm32"))]
    let path = CONFIG_PATH.to_owned();

    let content = macroquad::file::load_string(&path)
        .await
        .map_err(|error| format!("Could not load {CONFIG_PATH}: {error}"))?;
    Scene::from_a2ui_jsonl(&content).map_err(|error| error.to_string())
}

fn draw_background(time: f32) {
    clear_background(rgb(7, 12, 26));
    // GPU-drawn gradient and slow-moving ambient geometry. No HTML/CSS UI.
    for band in 0..48 {
        let t = band as f32 / 48.0;
        let r = (8.0 + 12.0 * t) as u8;
        let g = (13.0 + 18.0 * t) as u8;
        let b = (29.0 + 26.0 * t) as u8;
        draw_rectangle(0.0, VIEW_H * t, VIEW_W, VIEW_H / 48.0 + 1.0, rgb(r, g, b));
    }
    let drift = (time * 0.32).sin() * 40.0;
    draw_circle(1050.0 + drift, 35.0, 220.0, translucent(rgb(28, 77, 134), 0.10));
    draw_circle(900.0 - drift, -60.0, 120.0, translucent(rgb(70, 136, 200), 0.06));
}

fn draw_header() {
    draw_circle(109.0, 59.0, 11.0, rgb(82, 209, 222));
    draw_circle(109.0, 59.0, 5.0, rgb(15, 36, 56));
    draw_text("SMARTTVKIT", 132.0, 67.0, 26.0, rgb(239, 245, 251));

    draw_text("DISCOVER", 790.0, 65.0, 17.0, rgb(110, 222, 231));
    draw_text("LIVE TV", 932.0, 65.0, 17.0, rgb(164, 177, 199));
    draw_text("LIBRARY", 1063.0, 65.0, 17.0, rgb(164, 177, 199));
    draw_rectangle(789.0, 79.0, 97.0, 3.0, rgb(82, 209, 222));
}

fn draw_hero(scene: &Scene) {
    draw_text(&scene.headline, 108.0, 159.0, 52.0, rgb(245, 249, 255));
    draw_text(&scene.subtitle, 110.0, 198.0, 21.0, rgb(160, 174, 195));
}

fn draw_card_art(x: f32, y: f32, w: f32, h: f32, seed: usize) {
    let colors = [
        (rgb(33, 75, 118), rgb(65, 156, 187)),
        (rgb(65, 45, 109), rgb(150, 89, 163)),
        (rgb(21, 94, 100), rgb(74, 190, 157)),
        (rgb(126, 59, 54), rgb(206, 129, 69)),
        (rgb(44, 69, 123), rgb(96, 133, 221)),
        (rgb(63, 83, 70), rgb(122, 160, 92)),
        (rgb(72, 56, 106), rgb(146, 116, 202)),
        (rgb(24, 79, 120), rgb(74, 153, 196)),
        (rgb(113, 72, 49), rgb(204, 147, 89)),
        (rgb(61, 89, 112), rgb(144, 166, 194)),
    ];
    let (base, glow) = colors[seed % colors.len()];
    draw_rectangle(x, y, w, h, base);
    for band in 0..16 {
        let t = band as f32 / 16.0;
        draw_rectangle(
            x,
            y + t * h,
            w,
            h / 16.0 + 1.0,
            translucent(glow, t * 0.12),
        );
    }
    let float = (get_time() as f32 * 0.45 + seed as f32).sin() * 5.0;
    draw_circle(
        x + w * 0.74,
        y + h * 0.34 + float,
        55.0,
        translucent(glow, 0.32),
    );
    draw_circle(
        x + w * 0.74,
        y + h * 0.34 + float,
        30.0,
        translucent(rgb(248, 249, 255), 0.11),
    );
    for k in 0..5 {
        let shift = k as f32 * 14.0;
        draw_line(
            x + shift - 30.0,
            y + h,
            x + 115.0 + shift,
            y + 5.0,
            1.5,
            translucent(rgb(255, 255, 255), 0.07),
        );
    }
    draw_rectangle(
        x,
        y + h * 0.60,
        w,
        h * 0.40,
        translucent(rgb(6, 12, 28), 0.65),
    );
}

fn draw_one_card(
    title: &str,
    x: f32,
    y: f32,
    seed: usize,
    focused: bool,
    row: usize,
) {
    let raise = if focused { 6.0 } else { 0.0 };
    let card_y = y - raise;
    draw_rectangle(
        x + 4.0,
        card_y + 9.0,
        CARD_W,
        CARD_HEIGHT,
        translucent(BLACK, 0.20),
    );
    draw_card_art(x, card_y, CARD_W, CARD_HEIGHT, seed);
    draw_rectangle(
        x,
        card_y,
        CARD_W,
        CARD_HEIGHT,
        translucent(BLACK, if focused { 0.03 } else { 0.12 }),
    );
    let mut font_size = if row == 0 { 22.0 } else { 21.0 };
    while measure_text(title, None, font_size as u16, 1.0).width > CARD_W - 24.0
        && font_size > 13.0
    {
        font_size -= 1.0;
    }
    draw_text(
        title,
        x + 12.0,
        card_y + CARD_HEIGHT - 17.0,
        font_size,
        rgb(250, 250, 255),
    );
    draw_rectangle_lines(x, card_y, CARD_W, CARD_HEIGHT, 1.0, translucent(WHITE, 0.10));
}

fn draw_rails(scene: &Scene, nav: &Navigation) -> Option<(f32, f32)> {
    let mut focus: Option<(f32, f32)> = None;
    for (row, rail) in scene.rails.iter().take(2).enumerate() {
        let y = ROW_YS[row];
        draw_text(
            &rail.title,
            CARD_LEFT,
            y - 18.0,
            25.0,
            rgb(236, 243, 253),
        );
        // A rail can contain more than four cards; keep the focused item visible.
        let first_visible = if nav.row == row {
            nav.column.saturating_sub(3)
        } else {
            0
        };
        for (visible_index, (index, card)) in rail
            .cards
            .iter()
            .enumerate()
            .skip(first_visible)
            .take(4)
            .enumerate()
        {
            let x = CARD_LEFT + visible_index as f32 * (CARD_W + CARD_GAP);
            let selected = row == nav.row && index == nav.column;
            draw_one_card(
                &card.title,
                x,
                y,
                index + row * 5,
                selected,
                row,
            );
            if selected {
                focus = Some((x, y - 6.0));
            }
        }
    }
    focus
}

fn draw_focus(x: f32, y: f32, time: f32) {
    let glow = 0.68 + (time * 4.0).sin() * 0.12;
    draw_rectangle_lines(
        x - 4.0,
        y - 4.0,
        CARD_W + 8.0,
        CARD_HEIGHT + 8.0,
        4.0,
        translucent(rgb(100, 234, 241), glow),
    );
    draw_rectangle_lines(
        x - 7.0,
        y - 7.0,
        CARD_W + 14.0,
        CARD_HEIGHT + 14.0,
        1.0,
        translucent(rgb(176, 247, 253), 0.35),
    );
}

fn draw_details(scene: &Scene, nav: &Navigation) {
    if !nav.details_open {
        return;
    }
    if let Some(card) = nav.selected(scene) {
        draw_rectangle(0.0, 0.0, VIEW_W, VIEW_H, translucent(rgb(3, 8, 19), 0.91));
        draw_rectangle(217.0, 130.0, 846.0, 462.0, rgb(17, 29, 48));
        draw_rectangle(217.0, 130.0, 846.0, 6.0, rgb(89, 211, 221));
        draw_text("NOW SELECTED", 268.0, 206.0, 20.0, rgb(99, 224, 234));
        draw_text(&card.title, 268.0, 290.0, 48.0, rgb(248, 251, 255));
        draw_text(
            "Rendered by Rust from an A2UI v0.9.1 Text binding.",
            270.0,
            345.0,
            23.0,
            rgb(171, 185, 207),
        );
        draw_text(
            "Playback integration belongs to the next milestone.",
            270.0,
            385.0,
            20.0,
            rgb(139, 155, 180),
        );
        draw_rectangle(270.0, 462.0, 195.0, 50.0, rgb(61, 153, 172));
        draw_text("BACK  (ESC)", 297.0, 494.0, 21.0, WHITE);
    }
}

fn draw_footer(status: &str) {
    draw_rectangle(0.0, 678.0, VIEW_W, 42.0, translucent(rgb(5, 12, 23), 0.58));
    draw_text(
        "ARROWS: NAVIGATE    ENTER: DETAILS    ESC: BACK    R: RELOAD A2UI",
        108.0,
        704.0,
        16.0,
        rgb(159, 176, 197),
    );
    let summary = format!("{}  |  {} FPS", status, get_fps());
    let width = measure_text(&summary, None, 15, 1.0).width;
    draw_text(
        &summary,
        VIEW_W - width - 36.0,
        704.0,
        15.0,
        rgb(106, 205, 215),
    );
}

fn pointer_to_scene() -> (f32, f32) {
    let (x, y) = mouse_position();
    (
        x * VIEW_W / screen_width().max(1.0),
        y * VIEW_H / screen_height().max(1.0),
    )
}

#[macroquad::main(window_conf)]
async fn main() {
    let (mut scene, mut status) = match load_scene().await {
        Ok(scene) => (scene, "A2UI FILE LOADED".to_string()),
        Err(error) => {
            let fallback = Scene::from_a2ui_jsonl(BUNDLED_A2UI)
                .expect("bundled A2UI example must parse");
            (fallback, format!("FALLBACK: {error}"))
        }
    };

    let mut navigation = Navigation::default();
    let mut focus_x = CARD_LEFT;
    let mut focus_y = ROW_YS[0] - 6.0;

    loop {
        if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
            navigation.move_direction(Direction::Left, &scene);
        }
        if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
            navigation.move_direction(Direction::Right, &scene);
        }
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            navigation.move_direction(Direction::Up, &scene);
        }
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            navigation.move_direction(Direction::Down, &scene);
        }
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            navigation.enter(&scene);
        }
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Backspace) {
            navigation.back();
        }
        if is_key_pressed(KeyCode::R) {
            match load_scene().await {
                Ok(new_scene) => {
                    scene = new_scene;
                    navigation.reset_if_out_of_bounds(&scene);
                    status = "A2UI RELOADED".to_string();
                }
                Err(error) => {
                    status = format!("RELOAD FAILED: {error}");
                }
            }
        }
        if is_mouse_button_pressed(MouseButton::Left) && !navigation.details_open {
            let (mx, my) = pointer_to_scene();
            for (row, y) in ROW_YS.iter().enumerate().take(scene.rails.len().min(2)) {
                if my >= *y - 10.0 && my <= *y + CARD_HEIGHT {
                    for col in 0..4 {
                        let x = CARD_LEFT + col as f32 * (CARD_W + CARD_GAP);
                        if mx >= x && mx <= x + CARD_W {
                            let first = if navigation.row == row {
                                navigation.column.saturating_sub(3)
                            } else {
                                0
                            };
                            if first + col < scene.rails[row].cards.len() {
                                navigation.row = row;
                                navigation.column = first + col;
                            }
                        }
                    }
                }
            }
        }

        set_camera(&ui_camera());
        let time = get_time() as f32;
        draw_background(time);
        draw_header();
        draw_hero(&scene);
        if let Some((target_x, target_y)) = draw_rails(&scene, &navigation) {
            let lerp = 1.0 - (-get_frame_time().min(0.1) * 16.0).exp();
            focus_x += (target_x - focus_x) * lerp;
            focus_y += (target_y - focus_y) * lerp;
            if !navigation.details_open {
                draw_focus(focus_x, focus_y, time);
            }
        }
        draw_details(&scene, &navigation);
        draw_footer(&status);

        next_frame().await;
    }
}

#[cfg(test)]
mod camera_tests {
    use super::*;
    use macroquad::camera::Camera;

    /// Camera matrix coordinates are normalised device coordinates:
    /// top left = (-1, +1), bottom right = (+1, -1).
    /// This does not need a graphics context or a physical display.
    #[test]
    fn top_left_ui_coordinates_project_upright() {
        let matrix = ui_camera().matrix();
        let top_left = matrix.transform_point3(vec3(0.0, 0.0, 0.0));
        let bottom_right = matrix.transform_point3(vec3(VIEW_W, VIEW_H, 0.0));

        assert!((top_left.x + 1.0).abs() < 0.0001);
        assert!((top_left.y - 1.0).abs() < 0.0001);
        assert!((bottom_right.x - 1.0).abs() < 0.0001);
        assert!((bottom_right.y + 1.0).abs() < 0.0001);
    }
}
