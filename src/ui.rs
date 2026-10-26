use crate::models::{GeometryTier, RoutePhase};
use crate::parser::{execute_rust_command, TUTORIAL_STEPS};
use crate::world::{TileType, World};
use macroquad::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MouseTool {
    Build(&'static str),
    Excavate,
    Demolish,       // Destrói construções com reembolso e limpeza de rotas
    LinkRouteRaw,   // Conecta dois prédios com &mut clicando neles
    LinkRouteMutex, // Conecta dois prédios com Arc<Mutex> clicando neles
    PanCamera,      // Navegar pelo mapa arrastando o dedo na tela
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MobileTab {
    Build,    // Construções e ferramentas
    Upgrades, // Upgrades globais da categoria
    Academy,  // Lições e explicações didáticas de Rust
    Console,  // REPL, logs e status das entidades
}

pub struct AppState {
    pub world: World,
    pub camera_x: f32,
    pub camera_y: f32,
    pub camera_vx: f32,
    pub camera_vy: f32,
    pub zoom_tile_size: f32,
    pub active_tool: MouseTool,
    pub pending_route_bus: Option<u32>,
    pub pending_route_source: Option<u32>,
    pub command_input: String,
    pub console_log: String,
    pub tutorial_idx: usize,
    pub last_mouse: (f32, f32),
    // Gestos Touch Mobile & Pinça (Multi-Touch):
    pub pinch_prev_dist: Option<f32>,
    pub pinch_prev_mid: Option<Vec2>,
    pub touch_start_pos: Option<Vec2>,
    pub touch_start_time: f64,
    pub touch_has_dragged: bool,
    // Configurações Mobile & Economia de Bateria:
    pub battery_saver: bool,
    pub enable_safe_area: bool,
    pub active_mobile_tab: MobileTab,
    pub last_interaction_time: f64,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            camera_x: 0.0,
            camera_y: 0.0,
            camera_vx: 0.0,
            camera_vy: 0.0,
            zoom_tile_size: 64.0,
            active_tool: MouseTool::Build("byte_harvester"),
            pending_route_bus: None,
            pending_route_source: None,
            command_input: String::new(),
            console_log: "DICA: O carrinho anda estritamente na linha! Modo Touch & Mobile Ativo.".to_string(),
            tutorial_idx: 0,
            last_mouse: (0.0, 0.0),
            pinch_prev_dist: None,
            pinch_prev_mid: None,
            touch_start_pos: None,
            touch_start_time: 0.0,
            touch_has_dragged: false,
            battery_saver: true,        // 30 FPS no mobile para economizar bateria
            enable_safe_area: true,     // Protege notch e barra de gestos
            active_mobile_tab: MobileTab::Build,
            last_interaction_time: 0.0,
        }
    }
}

pub fn render_and_interact(app: &mut AppState) {
    let sw = screen_width();
    let sh = screen_height();
    let is_portrait = sw < sh;

    // Fundo Geral Cyber-Terminal (#0A0E14)
    clear_background(Color::new(0.04, 0.055, 0.08, 1.0));

    // Escala dinâmica DPI para celulares de alta resolução (Full HD, 2K, 3K)
    let ui_scale = if is_portrait {
        (sw / 440.0).clamp(1.0, 3.0)
    } else {
        (sh / 540.0).clamp(1.0, 2.2)
    };

    // Margens de Área Segura (Notch no topo e barra de navegação/gestos no rodapé)
    let safe_top = if app.enable_safe_area { 36.0 * ui_scale } else { 0.0 };
    let safe_bottom = if app.enable_safe_area { 28.0 * ui_scale } else { 0.0 };

    if is_portrait {
        render_portrait_layout(app, sw, sh, safe_top, safe_bottom, ui_scale);
    } else {
        render_landscape_layout(app, sw, sh, safe_top, safe_bottom, ui_scale);
    }
}

// =========================================================================
// LAYOUT HORIZONTAL (LANDSCAPE / DESKTOP / TABLET)
// =========================================================================
fn render_landscape_layout(app: &mut AppState, sw: f32, sh: f32, safe_top: f32, safe_bottom: f32, scale: f32) {
    let top_bar_h = 84.0 * scale;
    let bottom_repl_h = 88.0 * scale;
    let right_panel_w = (sw * 0.35).clamp(360.0 * scale, 520.0 * scale);

    let top_rect = Rect::new(8.0, safe_top + 4.0, sw - 16.0, top_bar_h);
    let map_rect = Rect::new(
        8.0,
        safe_top + top_bar_h + 8.0,
        sw - right_panel_w - 18.0,
        sh - safe_top - safe_bottom - top_bar_h - bottom_repl_h - 18.0,
    );
    let right_rect = Rect::new(
        sw - right_panel_w - 8.0,
        safe_top + top_bar_h + 8.0,
        right_panel_w,
        sh - safe_top - safe_bottom - top_bar_h - bottom_repl_h - 18.0,
    );
    let bottom_rect = Rect::new(
        8.0,
        sh - safe_bottom - bottom_repl_h - 4.0,
        sw - 16.0,
        bottom_repl_h,
    );

    draw_map_viewport(app, map_rect, scale);
    draw_top_toolbar(app, top_rect, scale);
    draw_right_education_panel(app, right_rect, scale);
    draw_bottom_terminal(app, bottom_rect, scale);
}

// =========================================================================
// LAYOUT VERTICAL (PORTRAIT / SMARTPHONE)
// =========================================================================
fn render_portrait_layout(app: &mut AppState, sw: f32, sh: f32, safe_top: f32, safe_bottom: f32, scale: f32) {
    let header_h = 66.0 * scale;
    let available_h = (sh - safe_top - safe_bottom - header_h - 16.0).max(100.0);
    // Mapa ganha 46% da altura útil
    let map_h = (available_h * 0.46).round();
    let panel_h = available_h - map_h - 8.0;

    let header_rect = Rect::new(6.0, safe_top + 4.0, sw - 12.0, header_h);
    let map_rect = Rect::new(6.0, safe_top + header_h + 6.0, sw - 12.0, map_h);
    let panel_rect = Rect::new(6.0, safe_top + header_h + 6.0 + map_h + 6.0, sw - 12.0, panel_h);

    draw_mobile_header(app, header_rect, scale);
    draw_map_viewport(app, map_rect, scale);
    draw_mobile_tabs_panel(app, panel_rect, scale);
}

// =========================================================================
// CABEÇALHO HORIZONTAL (LANDSCAPE)
// =========================================================================
fn draw_top_toolbar(app: &mut AppState, rect: Rect, scale: f32) {
    draw_panel_box(rect, " CENTRAL DE CONTROLE // STRUCT::FORGE ", Color::new(0.0, 0.8, 1.0, 1.0), 13.0 * scale);

    let bytes = app.world.resources.get("raw_bytes").copied().unwrap_or(0);
    let shards = app.world.resources.get("crypto_shards").copied().unwrap_or(0);
    let bytecode = app.world.resources.get("bytecode").copied().unwrap_or(0);

    let f_size = 15.0 * scale;
    draw_text(
        &format!("RawBytes: {}  |  Shards: {}  |  Bytecode: {}", bytes, shards, bytecode),
        rect.x + 12.0 * scale,
        rect.y + 26.0 * scale,
        f_size,
        Color::new(0.9, 0.95, 1.0, 1.0),
    );

    let eco_label = if app.battery_saver { "🔋 ECO: 30FPS" } else { "⚡ MAX: 60FPS" };
    let eco_col = if app.battery_saver { Color::new(0.3, 1.0, 0.5, 1.0) } else { Color::new(1.0, 0.8, 0.2, 1.0) };
    let eco_rect = Rect::new(rect.x + rect.w - 230.0 * scale, rect.y + 8.0 * scale, 108.0 * scale, 24.0 * scale);
    if draw_button(eco_rect, eco_label, eco_col, app.battery_saver, 12.0 * scale) {
        app.battery_saver = !app.battery_saver;
    }

    let notch_label = if app.enable_safe_area { "📱 NOTCH: ON" } else { "📱 NOTCH: OFF" };
    let notch_rect = Rect::new(rect.x + rect.w - 116.0 * scale, rect.y + 8.0 * scale, 108.0 * scale, 24.0 * scale);
    if draw_button(notch_rect, notch_label, Color::new(0.5, 0.8, 1.0, 1.0), app.enable_safe_area, 12.0 * scale) {
        app.enable_safe_area = !app.enable_safe_area;
    }

    let mut bx = rect.x + 8.0 * scale;
    let by = rect.y + 38.0 * scale;
    let bh = 34.0 * scale;

    let tools: &[(&str, MouseTool, Color)] = &[
        ("[1] Mineradora (25B)", MouseTool::Build("byte_harvester"), Color::new(0.0, 0.85, 1.0, 1.0)),
        ("[2] Vagão Bus (20B)", MouseTool::Build("packet_bus"), Color::new(1.0, 0.85, 0.1, 1.0)),
        ("[3] Torre Laser (35B)", MouseTool::Build("firewall_node"), Color::new(1.0, 0.35, 0.35, 1.0)),
        ("[4] Conversor (40B)", MouseTool::Build("transpiler"), Color::new(0.85, 0.35, 1.0, 1.0)),
        ("[5] Escavar (+5B)", MouseTool::Excavate, Color::new(0.6, 0.8, 0.6, 1.0)),
        ("[6] Ligar &mut", MouseTool::LinkRouteRaw, Color::new(0.2, 1.0, 0.5, 1.0)),
        ("[7] Ligar Mutex", MouseTool::LinkRouteMutex, Color::new(0.3, 0.9, 1.0, 1.0)),
        ("[8] Demolir (X)", MouseTool::Demolish, Color::new(1.0, 0.25, 0.25, 1.0)),
        ("[🖐] Mover", MouseTool::PanCamera, Color::new(0.8, 0.8, 0.9, 1.0)),
    ];

    for (label, tool, col) in tools {
        let bw = (label.len() as f32 * 7.2 * scale) + 12.0 * scale;
        let is_selected = app.active_tool == *tool;
        if draw_button(Rect::new(bx, by, bw, bh), label, *col, is_selected, 12.5 * scale) {
            app.active_tool = *tool;
            app.pending_route_bus = None;
            app.pending_route_source = None;
        }
        bx += bw + 5.0 * scale;
    }

    let upg_label = "★ Upgrade (10 Code)";
    let upg_w = 160.0 * scale;
    if rect.x + rect.w - upg_w - 6.0 > bx {
        if draw_button(
            Rect::new(rect.x + rect.w - upg_w - 6.0, by, upg_w, bh),
            upg_label,
            Color::new(1.0, 0.75, 0.1, 1.0),
            false,
            12.5 * scale,
        ) {
            match app.world.upgrade_global_blueprint("packet_bus", None) {
                Ok(msg) => app.console_log = format!("Mouse Action -> `impl packet_bus`: {}", msg),
                Err(e) => app.console_log = e,
            }
        }
    }
}

// =========================================================================
// CABEÇALHO VERTICAL (PORTRAIT)
// =========================================================================
fn draw_mobile_header(app: &mut AppState, rect: Rect, scale: f32) {
    draw_panel_box(rect, " STRUCT::FORGE // MOBILE HUD ", Color::new(0.0, 0.8, 1.0, 1.0), 12.0 * scale);

    let bytes = app.world.resources.get("raw_bytes").copied().unwrap_or(0);
    let shards = app.world.resources.get("crypto_shards").copied().unwrap_or(0);
    let bytecode = app.world.resources.get("bytecode").copied().unwrap_or(0);

    // Linha 1: Recursos em destaque
    draw_text(
        &format!("Bytes: {} | Shards: {} | Code: {}", bytes, shards, bytecode),
        rect.x + 8.0 * scale,
        rect.y + 26.0 * scale,
        15.0 * scale,
        Color::new(0.9, 0.95, 1.0, 1.0),
    );

    // Linha 2: Toggles Mobile
    let btn_h = 24.0 * scale;
    let eco_label = if app.battery_saver { "30 FPS" } else { "60 FPS" };
    let eco_col = if app.battery_saver { Color::new(0.3, 1.0, 0.5, 1.0) } else { Color::new(1.0, 0.8, 0.2, 1.0) };
    if draw_button(Rect::new(rect.x + 6.0 * scale, rect.y + 34.0 * scale, 96.0 * scale, btn_h), eco_label, eco_col, app.battery_saver, 12.0 * scale) {
        app.battery_saver = !app.battery_saver;
    }

    let notch_label = if app.enable_safe_area { "Notch: ON" } else { "Notch: OFF" };
    if draw_button(Rect::new(rect.x + 108.0 * scale, rect.y + 34.0 * scale, 108.0 * scale, btn_h), notch_label, Color::new(0.5, 0.8, 1.0, 1.0), app.enable_safe_area, 12.0 * scale) {
        app.enable_safe_area = !app.enable_safe_area;
    }

    if draw_button(Rect::new(rect.x + rect.w - 86.0 * scale, rect.y + 34.0 * scale, 80.0 * scale, btn_h), "Base", Color::new(0.3, 0.9, 1.0, 1.0), false, 12.0 * scale) {
        app.camera_x = 0.0;
        app.camera_y = 0.0;
    }
}

// =========================================================================
// PAINEL DE ABAS INFERIOR (PORTRAIT MOBILE)
// =========================================================================
fn draw_mobile_tabs_panel(app: &mut AppState, rect: Rect, scale: f32) {
    draw_panel_box(rect, "", Color::new(0.2, 0.6, 0.9, 1.0), 12.0 * scale);

    // Barra de Navegação das 4 Abas
    let tab_bar_h = 36.0 * scale;
    let tab_w = (rect.w - 12.0 * scale) / 4.0;
    let tabs = [
        ("Obras", MobileTab::Build),
        ("Upgrades", MobileTab::Upgrades),
        ("Academia", MobileTab::Academy),
        ("Console", MobileTab::Console),
    ];

    for (i, (label, tab)) in tabs.iter().enumerate() {
        let is_active = app.active_mobile_tab == *tab;
        let tab_rect = Rect::new(rect.x + 6.0 * scale + i as f32 * tab_w, rect.y + 4.0 * scale, tab_w - 3.0 * scale, tab_bar_h);
        let tab_col = if is_active { Color::new(0.0, 0.9, 1.0, 1.0) } else { Color::new(0.5, 0.6, 0.7, 1.0) };
        if draw_button(tab_rect, label, tab_col, is_active, 13.0 * scale) {
            app.active_mobile_tab = *tab;
        }
    }

    let content_rect = Rect::new(
        rect.x + 6.0 * scale,
        rect.y + tab_bar_h + 8.0 * scale,
        rect.w - 12.0 * scale,
        rect.h - tab_bar_h - 12.0 * scale,
    );

    match app.active_mobile_tab {
        MobileTab::Build => draw_mobile_build_tab(app, content_rect, scale),
        MobileTab::Upgrades => draw_mobile_upgrades_tab(app, content_rect, scale),
        MobileTab::Academy => draw_mobile_academy_tab(app, content_rect, scale),
        MobileTab::Console => draw_mobile_console_tab(app, content_rect, scale),
    }
}

fn draw_mobile_build_tab(app: &mut AppState, rect: Rect, scale: f32) {
    let tools: &[(&str, MouseTool, Color)] = &[
        ("[1] Mineradora (25B)", MouseTool::Build("byte_harvester"), Color::new(0.0, 0.85, 1.0, 1.0)),
        ("[2] Vagao Bus (20B)", MouseTool::Build("packet_bus"), Color::new(1.0, 0.85, 0.1, 1.0)),
        ("[3] Torre Laser (35B)", MouseTool::Build("firewall_node"), Color::new(1.0, 0.35, 0.35, 1.0)),
        ("[4] Conversor (40B)", MouseTool::Build("transpiler"), Color::new(0.85, 0.35, 1.0, 1.0)),
        ("[5] Escavar (+5B)", MouseTool::Excavate, Color::new(0.6, 0.8, 0.6, 1.0)),
        ("[6] Ligar &mut", MouseTool::LinkRouteRaw, Color::new(0.2, 1.0, 0.5, 1.0)),
        ("[7] Ligar Mutex", MouseTool::LinkRouteMutex, Color::new(0.3, 0.9, 1.0, 1.0)),
        ("[8] Demolir (X)", MouseTool::Demolish, Color::new(1.0, 0.25, 0.25, 1.0)),
        ("[Mover] Navegar", MouseTool::PanCamera, Color::new(0.7, 0.85, 1.0, 1.0)),
        ("[Base] Centralizar", MouseTool::PanCamera, Color::new(0.3, 0.9, 1.0, 1.0)),
    ];

    let cols = 2;
    let btn_w = (rect.w - 8.0 * scale) / cols as f32;
    // Otimização ergonômica: botões expandem para ocupar o espaço disponível vertical
    let btn_h = ((rect.h - 32.0 * scale) / 5.2).clamp(38.0 * scale, 75.0 * scale);

    for (idx, (label, tool, col)) in tools.iter().enumerate() {
        let r = idx / cols;
        let c = idx % cols;
        let bx = rect.x + c as f32 * btn_w;
        let by = rect.y + r as f32 * (btn_h + 6.0 * scale);

        if by + btn_h > rect.y + rect.h {
            break;
        }

        let is_selected = app.active_tool == *tool && idx != 9;
        if draw_button(Rect::new(bx, by, btn_w - 4.0 * scale, btn_h), label, *col, is_selected, 13.5 * scale) {
            if idx == 9 {
                app.camera_x = 0.0;
                app.camera_y = 0.0;
            } else {
                app.active_tool = *tool;
                app.pending_route_bus = None;
                app.pending_route_source = None;
            }
        }
    }
}

fn draw_mobile_upgrades_tab(app: &mut AppState, rect: Rect, scale: f32) {
    let cats = [
        ("packet_bus", "Vagão Packet Bus", "Velocidade + Capacidade"),
        ("byte_harvester", "Mineradora de Dados", "Taxa de Extração de Bytes"),
        ("firewall_node", "Torre Firewall Laser", "Alcance de Detecção e Dano"),
        ("transpiler", "Transpilador de Código", "Eficiência e Conversão"),
    ];

    let card_h = ((rect.h - 30.0 * scale) / 4.2).clamp(42.0 * scale, 80.0 * scale);
    for (i, (cat, name, desc)) in cats.iter().enumerate() {
        let cy = rect.y + i as f32 * (card_h + 8.0 * scale);
        if cy + card_h > rect.y + rect.h {
            break;
        }

        let bp = app.world.blueprints.registry.get(*cat);
        let tier_name = bp.map_or("BASIC", |b| b.tier.name());
        let col = bp.map_or(WHITE, |b| b.color);

        draw_rectangle(rect.x, cy, rect.w, card_h, Color::new(0.06, 0.08, 0.12, 0.9));
        draw_rectangle_lines(rect.x, cy, rect.w, card_h, 1.2, col);

        draw_text(name, rect.x + 8.0 * scale, cy + card_h * 0.42, 14.5 * scale, col);
        draw_text(&format!("Tier: {} | {}", tier_name, desc), rect.x + 8.0 * scale, cy + card_h * 0.78, 12.0 * scale, LIGHTGRAY);

        let upg_w = 120.0 * scale;
        let upg_h = card_h * 0.65;
        let upg_btn = Rect::new(rect.x + rect.w - upg_w - 6.0 * scale, cy + (card_h - upg_h) * 0.5, upg_w, upg_h);
        if draw_button(upg_btn, "impl (10 Code)", Color::new(1.0, 0.8, 0.2, 1.0), false, 12.5 * scale) {
            match app.world.upgrade_global_blueprint(cat, None) {
                Ok(msg) => app.console_log = format!("impl {}: {}", cat, msg),
                Err(err) => app.console_log = err,
            }
        }
    }
}

fn draw_mobile_academy_tab(app: &mut AppState, rect: Rect, scale: f32) {
    let step = &TUTORIAL_STEPS[app.tutorial_idx % TUTORIAL_STEPS.len()];
    let mut y = rect.y + 18.0 * scale;

    draw_text(step.title, rect.x + 6.0 * scale, y, 16.0 * scale, Color::new(0.2, 0.95, 1.0, 1.0));
    let next_btn = Rect::new(rect.x + rect.w - 110.0 * scale, rect.y, 104.0 * scale, 28.0 * scale);
    if draw_button(next_btn, "Próxima ▶", Color::new(0.2, 0.85, 1.0, 1.0), false, 12.5 * scale) {
        app.tutorial_idx = (app.tutorial_idx + 1) % TUTORIAL_STEPS.len();
    }
    y += 24.0 * scale;

    draw_text("• CONCEITO EM RUST REAL:", rect.x + 6.0 * scale, y, 13.0 * scale, Color::new(1.0, 0.85, 0.2, 1.0));
    y += 18.0 * scale;
    y = draw_wrapped_text(step.real_rust_meaning, rect.x + 8.0 * scale, y, rect.w - 16.0 * scale, 13.0 * scale, WHITE);
    y += 10.0 * scale;

    let exec_btn = Rect::new(
        rect.x + 6.0 * scale,
        (rect.y + rect.h - 40.0 * scale).max(y + 6.0 * scale),
        rect.w - 12.0 * scale,
        34.0 * scale,
    );
    if draw_button(exec_btn, &format!("▶ Rodar: {}", step.example_cmd), Color::new(0.2, 1.0, 0.5, 1.0), false, 13.5 * scale) {
        match execute_rust_command(&mut app.world, step.example_cmd) {
            Ok(msg) => app.console_log = msg,
            Err(err) => app.console_log = err,
        }
    }
}

fn draw_mobile_console_tab(app: &mut AppState, rect: Rect, scale: f32) {
    draw_text(
        &format!("borrow_checker> {}", app.world.last_borrow_event),
        rect.x + 6.0 * scale,
        rect.y + 18.0 * scale,
        13.5 * scale,
        Color::new(0.4, 0.9, 1.0, 1.0),
    );

    draw_text(
        &format!("output: {}", app.console_log),
        rect.x + 6.0 * scale,
        rect.y + 38.0 * scale,
        13.5 * scale,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );

    let mut cy = rect.y + 64.0 * scale;
    draw_text("UNIDADES EM CAMPO:", rect.x + 6.0 * scale, cy, 14.0 * scale, Color::new(0.7, 0.4, 1.0, 1.0));
    cy += 20.0 * scale;

    for st in app.world.structures.iter().take(6) {
        let inv: u32 = st.inventory.values().sum();
        let status = if let Some(r) = &st.route {
            if matches!(r.phase, RoutePhase::BorrowPanic(_)) {
                "PANIC E0499".to_string()
            } else {
                format!("Rota #{}->#{}", r.source_id, r.target_id)
            }
        } else {
            "Livre".to_string()
        };

        draw_text(
            &format!("#{:<2} {:<12} | Carga: {} | {}", st.id, st.category_id, inv, status),
            rect.x + 6.0 * scale,
            cy,
            12.5 * scale,
            WHITE,
        );
        cy += 18.0 * scale;
    }
}

// =========================================================================
// VIEWPORT DO MAPA (SUPORTA TOUCH DRAG, PANNING E ZOOM)
// =========================================================================
fn draw_map_viewport(app: &mut AppState, rect: Rect, scale: f32) {
    draw_panel_box(rect, " MAPA DA FÁBRICA // NAVEGAÇÃO 2D ", Color::new(0.2, 0.65, 0.9, 1.0), 12.0 * scale);

    let (mx, my) = mouse_position();
    let center_px_x = rect.x + rect.w * 0.5;
    let center_px_y = rect.y + rect.h * 0.5;

    // Botões Flutuantes de Zoom e Centralização no canto superior direito do mapa
    let z_btn_w = 42.0 * scale;
    let z_btn_h = 24.0 * scale;
    let zoom_in_rect = Rect::new(rect.x + rect.w - z_btn_w - 6.0 * scale, rect.y + 4.0 * scale, z_btn_w, z_btn_h);
    let zoom_out_rect = Rect::new(rect.x + rect.w - (z_btn_w * 2.0) - 10.0 * scale, rect.y + 4.0 * scale, z_btn_w, z_btn_h);
    let reset_zoom_rect = Rect::new(rect.x + rect.w - (z_btn_w * 3.0) - 14.0 * scale, rect.y + 4.0 * scale, z_btn_w, z_btn_h);
    let base_btn_rect = Rect::new(rect.x + rect.w - (z_btn_w * 4.0) - 18.0 * scale, rect.y + 4.0 * scale, z_btn_w, z_btn_h);

    let active_touches = touches();
    let mut touch_consumed = false;

    // 1. PINÇA MULTI-TOUCH / PINCH-TO-ZOOM E 2-FINGER PAN
    if active_touches.len() >= 2 {
        touch_consumed = true;
        let t0 = &active_touches[0];
        let t1 = &active_touches[1];
        let cur_dist = (t0.position - t1.position).length();
        let cur_mid = (t0.position + t1.position) * 0.5;

        if rect.contains(cur_mid) || rect.contains(t0.position) || rect.contains(t1.position) {
            if let (Some(prev_dist), Some(prev_mid)) = (app.pinch_prev_dist, app.pinch_prev_mid) {
                if prev_dist > 8.0 {
                    let ratio = (cur_dist / prev_dist).clamp(0.6, 1.6);
                    let old_zoom = app.zoom_tile_size;
                    let min_zoom = 20.0 * scale;
                    let max_zoom = 120.0 * scale;
                    let new_zoom = (old_zoom * ratio).clamp(min_zoom, max_zoom);

                    // Zoom centrado na posição exata dos dedos
                    let world_x_mid = app.camera_x + (cur_mid.x - center_px_x) / old_zoom;
                    let world_y_mid = app.camera_y + (cur_mid.y - center_px_y) / old_zoom;
                    app.camera_x = world_x_mid - (cur_mid.x - center_px_x) / new_zoom;
                    app.camera_y = world_y_mid - (cur_mid.y - center_px_y) / new_zoom;
                    app.zoom_tile_size = new_zoom;

                    // Deslocamento da câmera acompanhando os dois dedos
                    let pan_dx = cur_mid.x - prev_mid.x;
                    let pan_dy = cur_mid.y - prev_mid.y;
                    app.camera_x -= pan_dx / new_zoom;
                    app.camera_y -= pan_dy / new_zoom;
                    app.camera_vx = -pan_dx / new_zoom;
                    app.camera_vy = -pan_dy / new_zoom;
                }
            }
            app.pinch_prev_dist = Some(cur_dist);
            app.pinch_prev_mid = Some(cur_mid);
            app.touch_has_dragged = true; // Impede qualquer clique acidental ao soltar os dedos
            app.last_interaction_time = get_time();
        }
    } else {
        app.pinch_prev_dist = None;
        app.pinch_prev_mid = None;
    }

    // 2. ZOOM VIA MOUSE WHEEL (CENTRADÍSSIMO NO CURSOR PARA PC)
    let inside_map = rect.contains(vec2(mx, my));
    let (_, wheel_y) = mouse_wheel();
    if inside_map && wheel_y != 0.0 && active_touches.is_empty() {
        let old_zoom = app.zoom_tile_size;
        let factor = if wheel_y > 0.0 { 1.15 } else { 0.87 };
        let new_zoom = (old_zoom * factor).clamp(20.0 * scale, 120.0 * scale);
        let world_x_mouse = app.camera_x + (mx - center_px_x) / old_zoom;
        let world_y_mouse = app.camera_y + (my - center_px_y) / old_zoom;
        app.camera_x = world_x_mouse - (mx - center_px_x) / new_zoom;
        app.camera_y = world_y_mouse - (my - center_px_y) / new_zoom;
        app.zoom_tile_size = new_zoom;
        app.last_interaction_time = get_time();
    }

    // 3. TOQUE ÚNICO / MOUSE: PAN SUAVE COM INÉRCIA VS TAP LIMPO
    let pointer_pos = if let Some(t) = active_touches.first() {
        t.position
    } else {
        vec2(mx, my)
    };

    let pointer_inside = rect.contains(pointer_pos);
    let on_floating_btn = zoom_in_rect.contains(pointer_pos)
        || zoom_out_rect.contains(pointer_pos)
        || reset_zoom_rect.contains(pointer_pos)
        || base_btn_rect.contains(pointer_pos);

    let is_pressed = is_mouse_button_pressed(MouseButton::Left);
    let is_down = is_mouse_button_down(MouseButton::Left);
    let is_released = is_mouse_button_released(MouseButton::Left);
    let is_secondary_pan = is_mouse_button_down(MouseButton::Right) || is_mouse_button_down(MouseButton::Middle);
    let is_secondary_pan_pressed = is_mouse_button_pressed(MouseButton::Right) || is_mouse_button_pressed(MouseButton::Middle);

    if pointer_inside && !on_floating_btn && !touch_consumed {
        if is_pressed || is_secondary_pan_pressed {
            app.touch_start_pos = Some(pointer_pos);
            app.touch_start_time = get_time();
            app.touch_has_dragged = false;
            app.last_mouse = (pointer_pos.x, pointer_pos.y);
            app.camera_vx = 0.0;
            app.camera_vy = 0.0;
            app.last_interaction_time = get_time();
        }
    }

    if (is_down || is_secondary_pan) && !touch_consumed {
        if let Some(start_pos) = app.touch_start_pos {
            let drag_dist = (pointer_pos - start_pos).length();
            if drag_dist > 10.0 * scale {
                app.touch_has_dragged = true;
            }

            let should_pan = is_secondary_pan
                || app.active_tool == MouseTool::PanCamera
                || (app.touch_has_dragged && app.active_tool != MouseTool::Excavate);

            if should_pan {
                let dx = pointer_pos.x - app.last_mouse.0;
                let dy = pointer_pos.y - app.last_mouse.1;
                app.camera_x -= dx / app.zoom_tile_size;
                app.camera_y -= dy / app.zoom_tile_size;
                app.camera_vx = app.camera_vx * 0.4 - (dx / app.zoom_tile_size) * 0.6;
                app.camera_vy = app.camera_vy * 0.4 - (dy / app.zoom_tile_size) * 0.6;
                app.last_mouse = (pointer_pos.x, pointer_pos.y);
                app.last_interaction_time = get_time();
            } else if app.active_tool == MouseTool::Excavate && app.touch_has_dragged {
                let htx = (app.camera_x + (pointer_pos.x - center_px_x) / app.zoom_tile_size).round() as i32;
                let hty = (app.camera_y + (pointer_pos.y - center_px_y) / app.zoom_tile_size).round() as i32;
                if let Ok(msg) = app.world.excavate(htx, hty) {
                    app.console_log = format!("Touch Defrag -> executou `defrag!({}, {});` ({})", htx, hty, msg);
                }
            }
        }
    }

    let mut tap_executed = false;
    if is_released && !touch_consumed {
        if let Some(_start_pos) = app.touch_start_pos.take() {
            let press_duration = get_time() - app.touch_start_time;
            if !app.touch_has_dragged && press_duration < 0.45 && pointer_inside && !on_floating_btn {
                tap_executed = true;
            }
        }
    }

    // 4. FÍSICA DE INÉRCIA / MOMENTUM (DESLIZE SUAVE NATIVO MOBILE)
    if !is_down && !is_secondary_pan && active_touches.is_empty() {
        if app.camera_vx.abs() > 0.0001 || app.camera_vy.abs() > 0.0001 {
            app.camera_x += app.camera_vx;
            app.camera_y += app.camera_vy;
            app.camera_vx *= 0.90;
            app.camera_vy *= 0.90;
            if app.camera_vx.abs() < 0.001 { app.camera_vx = 0.0; }
            if app.camera_vy.abs() < 0.001 { app.camera_vy = 0.0; }
        }
    }

    app.camera_x = app.camera_x.clamp(-300.0, 300.0);
    app.camera_y = app.camera_y.clamp(-300.0, 300.0);

    let ts = app.zoom_tile_size;
    let center_px_x = rect.x + rect.w * 0.5;
    let center_px_y = rect.y + rect.h * 0.5;

    let cols_half = ((rect.w / ts) * 0.5) as i32 + 1;
    let rows_half = ((rect.h / ts) * 0.5) as i32 + 1;
    let cam_tile_x = app.camera_x.round() as i32;
    let cam_tile_y = app.camera_y.round() as i32;

    // 1. Grade do terreno
    for ty in (cam_tile_y - rows_half)..=(cam_tile_y + rows_half) {
        for tx in (cam_tile_x - cols_half)..=(cam_tile_x + cols_half) {
            let px = center_px_x + (tx as f32 - app.camera_x) * ts - ts * 0.5;
            let py = center_px_y + (ty as f32 - app.camera_y) * ts - ts * 0.5;

            if px + ts < rect.x + 2.0
                || px > rect.x + rect.w - 2.0
                || py + ts < rect.y + 34.0 * scale
                || py > rect.y + rect.h - 2.0
            {
                continue;
            }

            let tile = app.world.get_tile(tx, ty);
            let (bg_col, glyph, glyph_col) = match tile {
                TileType::EmptyFloor => (Color::new(0.06, 0.08, 0.12, 1.0), "·", Color::new(0.16, 0.22, 0.30, 1.0)),
                TileType::SolidRock => (Color::new(0.14, 0.16, 0.20, 1.0), "###", Color::new(0.38, 0.42, 0.48, 1.0)),
                TileType::IronVein => (Color::new(0.07, 0.16, 0.24, 1.0), "BYTE", Color::new(0.3, 0.85, 1.0, 0.85)),
                TileType::CrystalVein => (Color::new(0.16, 0.08, 0.26, 1.0), "SHRD", Color::new(0.9, 0.4, 1.0, 0.9)),
                TileType::BeastNest => (Color::new(0.25, 0.06, 0.08, 1.0), "NEST", Color::new(1.0, 0.2, 0.2, 0.9)),
            };

            draw_rectangle(px + 1.0, py + 1.0, ts - 2.0, ts - 2.0, bg_col);
            draw_rectangle_lines(px, py, ts, ts, 1.0, Color::new(0.10, 0.14, 0.20, 1.0));
            draw_text(glyph, px + ts * 0.16, py + ts * 0.58, ts * 0.32, glyph_col);
        }
    }

    // 2. Rotas Ortogonais Conectadas
    for st in &app.world.structures {
        if let Some(route) = &st.route {
            if route.path.len() >= 2 {
                let line_col = match route.phase {
                    RoutePhase::BorrowPanic(_) => Color::new(1.0, 0.15, 0.15, 0.95),
                    RoutePhase::WaitingMutex => Color::new(0.2, 0.9, 1.0, 0.8),
                    _ if route.use_mutex => Color::new(0.2, 0.85, 1.0, 0.65),
                    _ => Color::new(0.2, 1.0, 0.4, 0.65),
                };

                for window in route.path.windows(2) {
                    let p1 = window[0];
                    let p2 = window[1];
                    let x1 = center_px_x + (p1.0 as f32 - app.camera_x) * ts;
                    let y1 = center_px_y + (p1.1 as f32 - app.camera_y) * ts;
                    let x2 = center_px_x + (p2.0 as f32 - app.camera_x) * ts;
                    let y2 = center_px_y + (p2.1 as f32 - app.camera_y) * ts;

                    draw_line(x1, y1, x2, y2, 4.5 * scale, Color::new(line_col.r, line_col.g, line_col.b, 0.22));
                    draw_line(x1, y1, x2, y2, 2.2 * scale, line_col);
                    draw_circle(x1, y1, 2.5 * scale, line_col);
                }
                if let Some(last) = route.path.last() {
                    let lx = center_px_x + (last.0 as f32 - app.camera_x) * ts;
                    let ly = center_px_y + (last.1 as f32 - app.camera_y) * ts;
                    draw_circle(lx, ly, 2.5 * scale, line_col);
                }
            }
        }
    }

    // 3. Pré-visualização de rota ortogonal
    let hovered_tx = ((mx - center_px_x) / ts + app.camera_x).round() as i32;
    let hovered_ty = ((my - center_px_y) / ts + app.camera_y).round() as i32;

    if inside_map {
        if let Some(src_id) = app.pending_route_source {
            if let Some(src_st) = app.world.structures.iter().find(|s| s.id == src_id) {
                let preview_path = app.world.find_grid_path((src_st.x, src_st.y), (hovered_tx, hovered_ty));
                for window in preview_path.windows(2) {
                    let x1 = center_px_x + (window[0].0 as f32 - app.camera_x) * ts;
                    let y1 = center_px_y + (window[0].1 as f32 - app.camera_y) * ts;
                    let x2 = center_px_x + (window[1].0 as f32 - app.camera_x) * ts;
                    let y2 = center_px_y + (window[1].1 as f32 - app.camera_y) * ts;
                    draw_line(x1, y1, x2, y2, 2.5 * scale, Color::new(1.0, 0.9, 0.2, 0.75));
                }
            }
        }
    }

    // 4. Ondas de Pulso
    for pulse in &app.world.pulses {
        let px = center_px_x + (pulse.x - app.camera_x) * ts;
        let py = center_px_y + (pulse.y - app.camera_y) * ts;
        let alpha = 1.0 - (pulse.radius / pulse.max_radius).clamp(0.0, 1.0);
        let mut c = pulse.color;
        c.a = alpha;
        draw_circle_lines(px, py, pulse.radius * ts, 2.0 * scale, c);
    }

    // 5. Estruturas & Vagões
    for st in &app.world.structures {
        let cx = center_px_x + (st.render_x - app.camera_x) * ts;
        let cy = center_px_y + (st.render_y - app.camera_y) * ts;

        if cx + ts < rect.x || cx - ts > rect.x + rect.w || cy + ts < rect.y || cy - ts > rect.y + rect.h {
            continue;
        }

        if let Some(bp) = app.world.blueprints.registry.get(&st.category_id) {
            let active_tier = st.custom_tier_override.unwrap_or(bp.tier);
            let is_panic = matches!(st.route.as_ref().map(|r| &r.phase), Some(RoutePhase::BorrowPanic(_)));
            let is_waiting = matches!(st.route.as_ref().map(|r| &r.phase), Some(RoutePhase::WaitingMutex));

            let mut border_col = bp.color;
            if is_panic {
                border_col = Color::new(1.0, 0.1, 0.1, 1.0);
            } else if st.active_mut_borrower.is_some() {
                border_col = Color::new(0.2, 1.0, 0.4, 1.0);
            }

            let radius = ts * 0.42;
            draw_circle(cx, cy, radius, Color::new(0.05, 0.08, 0.12, 0.95));

            match active_tier {
                GeometryTier::Basic => {
                    draw_rectangle_lines(cx - radius, cy - radius, radius * 2.0, radius * 2.0, 2.5 * scale, border_col);
                }
                GeometryTier::Angular => {
                    draw_poly_lines(cx, cy, 4, radius * 1.1, 45.0, 3.0 * scale, border_col);
                }
                GeometryTier::HexCore => {
                    draw_poly_lines(cx, cy, 6, radius * 1.12, 0.0, 3.0 * scale, border_col);
                    draw_poly_lines(cx, cy, 6, radius * 0.85, 30.0, 1.5 * scale, Color::new(1.0, 0.9, 0.3, 0.9));
                }
            }

            let label = if is_panic {
                "E0499"
            } else if is_waiting {
                "WAIT"
            } else {
                bp.ascii_symbol
            };
            draw_text(label, cx - radius * 0.65, cy + 5.0 * scale, ts * 0.36, border_col);

            draw_text(
                &format!("#{}", st.id),
                cx - radius + 2.0,
                cy - radius + 11.0 * scale,
                13.0 * scale,
                Color::new(0.9, 0.95, 1.0, 0.9),
            );

            let inv_sum: u32 = st.inventory.values().sum();
            let cap = bp.effective_capacity(st.custom_tier_override).max(1);
            let ratio = (inv_sum as f32 / cap as f32).clamp(0.0, 1.0);
            let bar_w = radius * 1.6;
            draw_rectangle(cx - bar_w * 0.5, cy + radius - 5.0 * scale, bar_w, 4.0 * scale, Color::new(0.15, 0.2, 0.25, 1.0));
            draw_rectangle(cx - bar_w * 0.5, cy + radius - 5.0 * scale, bar_w * ratio, 4.0 * scale, bp.color);
        }
    }

    // 6. Glitches Nativos
    for beast in &app.world.beasts {
        let bx = center_px_x + (beast.x - app.camera_x) * ts;
        let by = center_px_y + (beast.y - app.camera_y) * ts;
        if bx >= rect.x && bx <= rect.x + rect.w && by >= rect.y && by <= rect.y + rect.h {
            draw_circle(bx, by, ts * 0.35, Color::new(0.8, 0.1, 0.1, 0.85));
            draw_circle_lines(bx, by, ts * 0.38, 2.0 * scale, RED);
            draw_text("wMw", bx - 12.0 * scale, by + 4.0 * scale, 15.0 * scale, WHITE);
            let hp_ratio = (beast.hp as f32 / beast.max_hp as f32).clamp(0.0, 1.0);
            draw_rectangle(bx - 14.0 * scale, by - 18.0 * scale, 28.0 * scale * hp_ratio, 4.0 * scale, RED);
        }
    }

    // 7. Projéteis Visíveis
    for p in &app.world.projectiles {
        let px = center_px_x + (p.x - app.camera_x) * ts;
        let py = center_px_y + (p.y - app.camera_y) * ts;
        draw_circle(px, py, 5.5 * scale, p.color);
        draw_circle(px, py, 3.0 * scale, WHITE);
    }

    // 8. Interação de Toque / Mouse
    if pointer_inside && !on_floating_btn && app.active_tool != MouseTool::PanCamera {
        let h_px = center_px_x + (hovered_tx as f32 - app.camera_x) * ts - ts * 0.5;
        let h_py = center_px_y + (hovered_ty as f32 - app.camera_y) * ts - ts * 0.5;

        if app.active_tool == MouseTool::Demolish {
            draw_rectangle_lines(h_px, h_py, ts, ts, 2.5 * scale, Color::new(1.0, 0.2, 0.2, 0.9));
            draw_text("X", h_px + ts * 0.38, h_py + ts * 0.62, ts * 0.45, RED);
        } else {
            draw_rectangle_lines(h_px, h_py, ts, ts, 2.0 * scale, Color::new(1.0, 1.0, 1.0, 0.65));
        }

        if tap_executed {
            match app.active_tool {
                MouseTool::Build(cat) => {
                    if let Ok(id) = app.world.spawn_structure(cat, hovered_tx, hovered_ty) {
                        app.console_log = format!(
                            "Touch/Click Build -> executou `spawn!({}, {}, {});` (Criou #{})",
                            cat, hovered_tx, hovered_ty, id
                        );
                    }
                }
                MouseTool::Excavate => {
                    if let Ok(msg) = app.world.excavate(hovered_tx, hovered_ty) {
                        app.console_log = format!(
                            "Touch/Click Defrag -> executou `defrag!({}, {});` ({})",
                            hovered_tx, hovered_ty, msg
                        );
                    }
                }
                MouseTool::Demolish => {
                    if let Some(target_st) = app
                        .world
                        .structures
                        .iter()
                        .find(|s| s.x == hovered_tx && s.y == hovered_ty)
                    {
                        let tid = target_st.id;
                        if let Ok(msg) = app.world.demolish_structure(tid) {
                            app.console_log = format!("Touch/Click Demolish -> executou `destroy!({});`: {}", tid, msg);
                        }
                    }
                }
                MouseTool::LinkRouteRaw | MouseTool::LinkRouteMutex => {
                    let use_mutex = app.active_tool == MouseTool::LinkRouteMutex;
                    if let Some(clicked_st) = app
                        .world
                        .structures
                        .iter()
                        .find(|s| s.x == hovered_tx && s.y == hovered_ty)
                    {
                        let clicked_id = clicked_st.id;

                        if clicked_st.category_id == "packet_bus" && app.pending_route_bus.is_none() && app.pending_route_source.is_none() {
                            app.pending_route_bus = Some(clicked_id);
                            app.console_log = format!("Vagão #{} selecionado! Agora toque na Origem (ex: Mineradora).", clicked_id);
                        } else if app.pending_route_source.is_none() {
                            app.pending_route_source = Some(clicked_id);
                            app.console_log = format!("Origem #{} selecionada! Agora toque no Destino (ex: Conversor/Base).", clicked_id);
                        } else {
                            let src_id = app.pending_route_source.unwrap();
                            let dst_id = clicked_id;

                            if src_id != dst_id {
                                let bus_id_opt = app.pending_route_bus.or_else(|| {
                                    app.world.structures.iter().find(|s| s.category_id == "packet_bus" && s.route.is_none()).map(|s| s.id)
                                });

                                if let Some(bus_id) = bus_id_opt {
                                    let mut_flag = if use_mutex { "" } else { "mut " };
                                    let macro_name = if use_mutex { "route_mutex" } else { "route_borrow" };
                                    let cmd = format!("{}!({} => {}{}, {});", macro_name, bus_id, mut_flag, src_id, dst_id);

                                    match execute_rust_command(&mut app.world, &cmd) {
                                        Ok(msg) => {
                                            app.console_log = format!("Touch/Click Route -> executou: `{}` ({})", cmd, msg);
                                        }
                                        Err(err) => app.console_log = err,
                                    }
                                } else {
                                    app.console_log = "ERRO: Nenhum Packet Bus disponível para vincular a esta rota!".to_string();
                                }
                            }
                            app.pending_route_source = None;
                            app.pending_route_bus = None;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // 9. Tooltip Informativo
    if pointer_inside && !on_floating_btn {
        if let Some(st) = app.world.structures.iter().find(|s| s.x == hovered_tx && s.y == hovered_ty) {
            if let Some(bp) = app.world.blueprints.registry.get(&st.category_id) {
                let inv_total: u32 = st.inventory.values().sum();
                let cap = bp.effective_capacity(st.custom_tier_override);
                let lock_txt = match st.active_mut_borrower {
                    Some(id) => format!("Emprestado (&mut #{})", id),
                    None => "Livre".to_string(),
                };

                let tip_w = (280.0 * scale).min(rect.w - 16.0);
                let tip_h = 86.0 * scale;
                let tx_box = (pointer_pos.x + 12.0).min(rect.x + rect.w - tip_w - 6.0);
                let ty_box = (pointer_pos.y + 12.0).min(rect.y + rect.h - tip_h - 6.0);

                let tip_border = if app.active_tool == MouseTool::Demolish { RED } else { bp.color };
                draw_rectangle(tx_box, ty_box, tip_w, tip_h, Color::new(0.03, 0.05, 0.09, 0.96));
                draw_rectangle_lines(tx_box, ty_box, tip_w, tip_h, 1.5 * scale, tip_border);

                if app.active_tool == MouseTool::Demolish {
                    draw_text(&format!("DEMOLIR #{} {}", st.id, bp.display_name), tx_box + 8.0 * scale, ty_box + 22.0 * scale, 14.5 * scale, RED);
                    draw_text(&format!("+{} Bytes, +{} Shards", bp.cost_bytes, bp.cost_shards), tx_box + 8.0 * scale, ty_box + 44.0 * scale, 12.5 * scale, WHITE);
                } else {
                    draw_text(&format!("#{} {}", st.id, bp.display_name), tx_box + 8.0 * scale, ty_box + 20.0 * scale, 14.5 * scale, bp.color);
                    draw_text(&format!("Carga: {}/{} | Status: {}", inv_total, cap, lock_txt), tx_box + 8.0 * scale, ty_box + 40.0 * scale, 12.5 * scale, WHITE);
                }
            }
        }
    }

    // 10. CONTROLES FLUTUANTES DA CÂMERA (DESENHADOS NO TOPO DE TUDO)
    draw_rectangle(rect.x + 1.0, rect.y + 1.0, rect.w - 2.0, 32.0 * scale, Color::new(0.04, 0.06, 0.09, 0.94));
    draw_line(rect.x, rect.y + 32.0 * scale, rect.x + rect.w, rect.y + 32.0 * scale, 1.0, Color::new(0.12, 0.22, 0.32, 1.0));

    let zoom_pct = ((app.zoom_tile_size / (64.0 * scale)) * 100.0).round() as i32;
    draw_text(
        &format!("POS: (X:{:+3.0}, Y:{:+3.0}) | ZOOM: {}%", app.camera_x, app.camera_y, zoom_pct),
        rect.x + 8.0 * scale,
        rect.y + 20.0 * scale,
        12.0 * scale,
        Color::new(0.3, 0.9, 1.0, 0.95),
    );

    if draw_button(zoom_in_rect, "+", Color::new(0.3, 0.9, 1.0, 1.0), false, 15.0 * scale) {
        app.zoom_tile_size = (app.zoom_tile_size * 1.22).clamp(20.0 * scale, 120.0 * scale);
        app.last_interaction_time = get_time();
    }
    if draw_button(zoom_out_rect, "-", Color::new(0.3, 0.9, 1.0, 1.0), false, 15.0 * scale) {
        app.zoom_tile_size = (app.zoom_tile_size * 0.82).clamp(20.0 * scale, 120.0 * scale);
        app.last_interaction_time = get_time();
    }
    if draw_button(reset_zoom_rect, "1:1", Color::new(0.4, 0.85, 1.0, 1.0), false, 11.5 * scale) {
        app.zoom_tile_size = 64.0 * scale;
        app.last_interaction_time = get_time();
    }
    if draw_button(base_btn_rect, "Base", Color::new(0.3, 1.0, 0.6, 1.0), false, 11.5 * scale) {
        app.camera_x = 0.0;
        app.camera_y = 0.0;
        app.camera_vx = 0.0;
        app.camera_vy = 0.0;
        app.last_interaction_time = get_time();
    }
}

// =========================================================================
// PAINEL LATERAL DIREITO (LANDSCAPE)
// =========================================================================
fn draw_right_education_panel(app: &mut AppState, rect: Rect, scale: f32) {
    let top_h = (rect.h * 0.67).round();
    let bot_h = rect.h - top_h - 8.0 * scale;

    let edu_rect = Rect::new(rect.x, rect.y, rect.w, top_h);
    let db_rect = Rect::new(rect.x, rect.y + top_h + 8.0 * scale, rect.w, bot_h);

    draw_panel_box(edu_rect, " ACADEMIA RUST (EXPLICAÇÃO PASSO A PASSO) ", Color::new(0.0, 0.9, 0.7, 1.0), 12.0 * scale);

    let step = &TUTORIAL_STEPS[app.tutorial_idx % TUTORIAL_STEPS.len()];
    let mut y = edu_rect.y + 24.0 * scale;

    draw_text(step.title, edu_rect.x + 12.0 * scale, y, 15.0 * scale, Color::new(0.2, 0.95, 1.0, 1.0));
    let next_btn = Rect::new(edu_rect.x + edu_rect.w - 110.0 * scale, edu_rect.y + 6.0 * scale, 102.0 * scale, 22.0 * scale);
    if draw_button(next_btn, "Próxima ▶", Color::new(0.2, 0.85, 1.0, 1.0), false, 11.0 * scale) {
        app.tutorial_idx = (app.tutorial_idx + 1) % TUTORIAL_STEPS.len();
    }
    y += 20.0 * scale;

    draw_text("• O QUE É NO RUST REAL?", edu_rect.x + 12.0 * scale, y, 12.5 * scale, Color::new(1.0, 0.85, 0.2, 1.0));
    y += 15.0 * scale;
    y = draw_wrapped_text(step.real_rust_meaning, edu_rect.x + 14.0 * scale, y, edu_rect.w - 28.0 * scale, 11.5 * scale, WHITE);
    y += 6.0 * scale;

    draw_text("• O QUE FAZ NA SUA FÁBRICA?", edu_rect.x + 12.0 * scale, y, 12.5 * scale, Color::new(0.3, 1.0, 0.5, 1.0));
    y += 15.0 * scale;
    y = draw_wrapped_text(step.game_effect, edu_rect.x + 14.0 * scale, y, edu_rect.w - 28.0 * scale, 11.5 * scale, LIGHTGRAY);
    y += 6.0 * scale;

    let exec_rect = Rect::new(edu_rect.x + 12.0 * scale, edu_rect.y + edu_rect.h - 34.0 * scale, edu_rect.w - 24.0 * scale, 26.0 * scale);

    draw_text("• ENTENDENDO CADA PALAVRA DO COMANDO:", edu_rect.x + 12.0 * scale, y, 12.5 * scale, Color::new(0.85, 0.45, 1.0, 1.0));
    y += 15.0 * scale;
    for line in step.token_breakdown {
        if y + 14.0 * scale < exec_rect.y - 4.0 * scale {
            draw_text(line, edu_rect.x + 14.0 * scale, y, 11.5 * scale, Color::new(0.8, 0.9, 1.0, 1.0));
            y += 14.0 * scale;
        }
    }

    let btn_label = format!("▶ Executar Agora: {}", step.example_cmd);
    if draw_button(exec_rect, &btn_label, Color::new(0.2, 1.0, 0.5, 1.0), false, 12.0 * scale) {
        match execute_rust_command(&mut app.world, step.example_cmd) {
            Ok(msg) => app.console_log = msg,
            Err(err) => app.console_log = err,
        }
    }

    draw_panel_box(db_rect, " ESTRUTURAS ATIVAS & BORROW CHECKER ", Color::new(0.7, 0.4, 1.0, 1.0), 12.5 * scale);
    let mut dy = db_rect.y + 24.0 * scale;

    for st in app.world.structures.iter().take(8) {
        let inv: u32 = st.inventory.values().sum();
        let status = if let Some(r) = &st.route {
            match &r.phase {
                RoutePhase::BorrowPanic(_) => "ERRO E0499 (Travado!)".to_string(),
                RoutePhase::WaitingMutex => format!("Fila Mutex (#{}->#{})", r.source_id, r.target_id),
                _ => {
                    let mode = if r.use_mutex { "Mutex" } else { "&mut" };
                    format!("Rota {} (#{}->#{})", mode, r.source_id, r.target_id)
                }
            }
        } else if let Some(holder) = st.active_mut_borrower {
            format!("Emprestado para &mut #{}", holder)
        } else {
            "Operando Livre".to_string()
        };

        let col = if status.contains("E0499") {
            Color::new(1.0, 0.3, 0.3, 1.0)
        } else if st.active_mut_borrower.is_some() {
            Color::new(0.3, 1.0, 0.5, 1.0)
        } else {
            WHITE
        };

        draw_text(
            &format!("#{:<2} {:<14} | Carga: {:<2} | {}", st.id, st.category_id, inv, status),
            db_rect.x + 10.0 * scale,
            dy,
            13.0 * scale,
            col,
        );
        dy += 17.0 * scale;
    }
}

// =========================================================================
// TERMINAL INFERIOR (LANDSCAPE)
// =========================================================================
fn draw_bottom_terminal(app: &mut AppState, rect: Rect, scale: f32) {
    draw_panel_box(
        rect,
        " TERMINAL RUST SIMULADO // EVENTOS & LOGS ",
        Color::new(0.2, 1.0, 0.5, 1.0),
        12.5 * scale,
    );

    draw_text(
        &format!("borrow_checker> {}", app.world.last_borrow_event),
        rect.x + 12.0 * scale,
        rect.y + 24.0 * scale,
        14.0 * scale,
        Color::new(0.4, 0.9, 1.0, 1.0),
    );

    draw_text(
        &format!("rust_console  > {}█", app.command_input),
        rect.x + 12.0 * scale,
        rect.y + 44.0 * scale,
        16.0 * scale,
        Color::new(0.3, 1.0, 0.4, 1.0),
    );

    draw_text(
        &format!("output        : {}", app.console_log),
        rect.x + 12.0 * scale,
        rect.y + 64.0 * scale,
        14.0 * scale,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );
}

// =========================================================================
// WIDGETS AUXILIARES COM ESCALA
// =========================================================================
fn draw_panel_box(rect: Rect, title: &str, border_color: Color, title_size: f32) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.05, 0.07, 0.11, 0.96));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border_color);
    if !title.is_empty() {
        draw_text(title, rect.x + 8.0, rect.y + title_size + 2.0, title_size, border_color);
    }
}

fn draw_button(rect: Rect, text: &str, color: Color, active: bool, font_size: f32) -> bool {
    let (mx, my) = mouse_position();
    let mouse_pos = vec2(mx, my);
    let active_touches = touches();
    let touch_pos = active_touches.first().map(|t| t.position);
    let pos = touch_pos.unwrap_or(mouse_pos);
    let hovered = rect.contains(pos);

    let mouse_clicked = rect.contains(mouse_pos) && is_mouse_button_pressed(MouseButton::Left);
    let touch_clicked = active_touches.iter().any(|t| (t.phase == TouchPhase::Started || t.phase == TouchPhase::Ended) && rect.contains(t.position));
    let clicked = mouse_clicked || touch_clicked;

    let bg = if active {
        Color::new(color.r * 0.35, color.g * 0.35, color.b * 0.35, 0.95)
    } else if hovered {
        Color::new(color.r * 0.22, color.g * 0.22, color.b * 0.22, 0.85)
    } else {
        Color::new(0.06, 0.08, 0.12, 0.85)
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, if active { 2.0 } else { 1.2 }, color);

    let dims = measure_text(text, None, font_size as u16, 1.0);
    let tx = rect.x + (rect.w - dims.width) * 0.5;
    let ty = rect.y + (rect.h + dims.height * 0.6) * 0.5;
    draw_text(
        text,
        tx.max(rect.x + 3.0),
        ty,
        font_size,
        if active || hovered { WHITE } else { Color::new(0.85, 0.9, 0.95, 1.0) },
    );

    clicked
}

fn draw_wrapped_text(text: &str, x: f32, mut y: f32, max_w: f32, font_size: f32, color: Color) -> f32 {
    let words = text.split_whitespace();
    let mut current_line = String::new();

    for word in words {
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };
        let dims = measure_text(&test_line, None, font_size as u16, 1.0);
        if dims.width > max_w && !current_line.is_empty() {
            draw_text(&current_line, x, y, font_size, color);
            y += font_size + 4.0;
            current_line = word.to_string();
        } else {
            current_line = test_line;
        }
    }
    if !current_line.is_empty() {
        draw_text(&current_line, x, y, font_size, color);
        y += font_size + 4.0;
    }
    y
}
