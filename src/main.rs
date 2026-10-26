mod models;
mod parser;
mod ui;
mod world;

use macroquad::prelude::*;
use ui::{render_and_interact, AppState, MouseTool};

fn window_conf() -> Conf {
    Conf {
        window_title: "STRUCT::FORGE // CYBER-TERMINAL EDITION".to_owned(),
        window_width: 1440,
        window_height: 860,
        window_resizable: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = AppState::new();

    loop {
        let frame_start = get_time();
        let dt = get_frame_time().min(0.1);

        // Rastreamento de interação para otimização de sono / bateria inteligente
        if is_mouse_button_down(MouseButton::Left)
            || is_mouse_button_down(MouseButton::Right)
            || is_mouse_button_down(MouseButton::Middle)
            || is_key_down(KeyCode::Left)
            || is_key_down(KeyCode::Right)
            || is_key_down(KeyCode::Up)
            || is_key_down(KeyCode::Down)
            || get_char_pressed().is_some()
        {
            app.last_interaction_time = frame_start;
        }

        // Atalhos rápidos no teclado físico / bluetooth
        if is_key_pressed(KeyCode::Tab) {
            app.tutorial_idx = (app.tutorial_idx + 1) % parser::TUTORIAL_STEPS.len();
        }
        if is_key_pressed(KeyCode::F1) {
            app.active_tool = MouseTool::Build("byte_harvester");
        }
        if is_key_pressed(KeyCode::F2) {
            app.active_tool = MouseTool::Build("packet_bus");
        }
        if is_key_pressed(KeyCode::F3) {
            app.active_tool = MouseTool::Build("firewall_node");
        }
        if is_key_pressed(KeyCode::F4) {
            app.active_tool = MouseTool::Build("transpiler");
        }
        if is_key_pressed(KeyCode::F5) {
            app.active_tool = MouseTool::Excavate;
        }
        if is_key_pressed(KeyCode::F6) || (is_key_pressed(KeyCode::X) && app.command_input.is_empty()) {
            app.active_tool = MouseTool::Demolish;
        }
        if is_key_pressed(KeyCode::F7) {
            app.active_tool = MouseTool::PanCamera;
        }

        if is_key_pressed(KeyCode::Backspace) {
            app.command_input.pop();
        }
        if is_key_pressed(KeyCode::Enter) && !app.command_input.is_empty() {
            let cmd = app.command_input.clone();
            app.command_input.clear();
            match parser::execute_rust_command(&mut app.world, &cmd) {
                Ok(msg) => app.console_log = msg,
                Err(err) => app.console_log = err,
            }
        }

        while let Some(c) = get_char_pressed() {
            if !c.is_control() {
                app.command_input.push(c);
            }
        }

        // Atualiza física contínua (projéteis, deslizamento dos vagões e Borrow Checker)
        app.world.update_continuous(dt);

        // Renderiza a interface moderna responsiva (Vertical e Horizontal)
        render_and_interact(&mut app);

        // =========================================================================
        // CONTROLE DE FRAMERATE E PACING DE BATERIA (ZERO BATTERY LEAK)
        // =========================================================================
        let now = get_time();
        let is_idle = (now - app.last_interaction_time) > 4.0 && app.world.projectiles.is_empty();

        let target_fps = if app.battery_saver {
            if is_idle { 20.0 } else { 30.0 }
        } else {
            60.0
        };

        let target_frame_duration = 1.0 / target_fps;
        let frame_work_duration = get_time() - frame_start;
        if frame_work_duration < target_frame_duration {
            let sleep_seconds = (target_frame_duration - frame_work_duration).max(0.001);
            std::thread::sleep(std::time::Duration::from_secs_f64(sleep_seconds));
        }

        next_frame().await;
    }
}
