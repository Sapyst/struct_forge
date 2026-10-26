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
    pub zoom_tile_size: f32,
    pub active_tool: MouseTool,
    pub pending_route_bus: Option<u32>,
    pub pending_route_source: Option<u32>,
    pub command_input: String,
    pub console_log: String,
    pub tutorial_idx: usize,
    pub is_panning: bool,
    pub last_mouse: (f32, f32),
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
            zoom_tile_size: 44.0,
            active_tool: MouseTool::Build("byte_harvester"),
            pending_route_bus: None,
            pending_route_source: None,
            command_input: String::new(),
            console_log: "DICA: O carrinho anda estritamente na linha! Use [8] Demolir para apagar estruturas.".to_string(),
            tutorial_idx: 0,
            is_panning: false,
            last_mouse: (0.0, 0.0),
            battery_saver: true,        // Ativo por padrão: 30 FPS no mobile para economizar bateria
            enable_safe_area: true,     // Ativo por padrão: protege notch e barra de gestos
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

    // Margens de Área Segura (Notch no topo e barra de navegação/gestos no rodapé)
    let safe_top = if app.enable_safe_area { 34.0 } else { 0.0 };
    let safe_bottom = if app.enable_safe_area { 26.0 } else { 0.0 };

    if is_portrait {
        render_portrait_layout(app, sw, sh, safe_top, safe_bottom);
    } else {
        render_landscape_layout(app, sw, sh, safe_top, safe_bottom);
    }
}

// =========================================================================
// LAYOUT HORIZONTAL (LANDSCAPE / DESKTOP / TABLET)
// =========================================================================
fn render_landscape_layout(app: &mut AppState, sw: f32, sh: f32, safe_top: f32, safe_bottom: f32) {
    let top_bar_h = 84.0;
    let bottom_repl_h = 88.0;
    let right_panel_w = (sw * 0.35).clamp(360.0, 500.0);

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

    // 1. Renderiza Mapa 2D Interativo
    draw_map_viewport(app, map_rect);

    // 2. Barra Superior: Recursos + Botões de Ferramentas + Bateria
    draw_top_toolbar(app, top_rect);

    // 3. Painel Lateral Direito: Academia Rust + Inspetor
    draw_right_education_panel(app, right_rect);

    // 4. Terminal Simulado Inferior
    draw_bottom_terminal(app, bottom_rect);
}

// =========================================================================
// LAYOUT VERTICAL (PORTRAIT / SMARTPHONE)
// =========================================================================
fn render_portrait_layout(app: &mut AppState, sw: f32, sh: f32, safe_top: f32, safe_bottom: f32) {
    let header_h = 68.0;
    let available_h = (sh - safe_top - safe_bottom - header_h - 16.0).max(100.0);
    // Mapa ganha 50% da altura da tela vertical
    let map_h = (available_h * 0.50).round();
    let panel_h = available_h - map_h - 8.0;

    let header_rect = Rect::new(6.0, safe_top + 4.0, sw - 12.0, header_h);
    let map_rect = Rect::new(6.0, safe_top + header_h + 8.0, sw - 12.0, map_h);
    let panel_rect = Rect::new(6.0, safe_top + header_h + 8.0 + map_h + 6.0, sw - 12.0, panel_h);

    // 1. Cabeçalho Superior Mobile com Safe Area
    draw_mobile_header(app, header_rect);

    // 2. Viewport do Mapa 2D no centro
    draw_map_viewport(app, map_rect);

    // 3. Painel de Abas Inferior (Ferramentas, Upgrades, Academia, Console)
    draw_mobile_tabs_panel(app, panel_rect);
}

// =========================================================================
// CABEÇALHO HORIZONTAL (LANDSCAPE)
// =========================================================================
fn draw_top_toolbar(app: &mut AppState, rect: Rect) {
    draw_panel_box(
        rect,
        " CENTRAL DE CONTROLE // STRUCT::FORGE ",
        Color::new(0.0, 0.8, 1.0, 1.0),
    );

    let bytes = app.world.resources.get("raw_bytes").copied().unwrap_or(0);
    let shards = app.world.resources.get("crypto_shards").copied().unwrap_or(0);
    let bytecode = app.world.resources.get("bytecode").copied().unwrap_or(0);

    // Linha 1: Recursos + Modos Rápidos de Energia e Notch
    draw_text(
        &format!("RawBytes: {}  |  Shards: {}  |  Bytecode: {}", bytes, shards, bytecode),
        rect.x + 12.0,
        rect.y + 28.0,
        17.0,
        Color::new(0.9, 0.95, 1.0, 1.0),
    );

    // Botões de Economia de Bateria e Safe Area no canto superior direito
    let eco_label = if app.battery_saver { "🔋 ECO: 30FPS" } else { "⚡ MAX: 60FPS" };
    let eco_col = if app.battery_saver { Color::new(0.3, 1.0, 0.5, 1.0) } else { Color::new(1.0, 0.8, 0.2, 1.0) };
    let eco_rect = Rect::new(rect.x + rect.w - 230.0, rect.y + 8.0, 108.0, 24.0);
    if draw_button(eco_rect, eco_label, eco_col, app.battery_saver) {
        app.battery_saver = !app.battery_saver;
    }

    let notch_label = if app.enable_safe_area { "📱 NOTCH: ON" } else { "📱 NOTCH: OFF" };
    let notch_rect = Rect::new(rect.x + rect.w - 116.0, rect.y + 8.0, 108.0, 24.0);
    if draw_button(notch_rect, notch_label, Color::new(0.5, 0.8, 1.0, 1.0), app.enable_safe_area) {
        app.enable_safe_area = !app.enable_safe_area;
    }

    // Linha 2: Ferramentas de Construção e Ações
    let mut bx = rect.x + 8.0;
    let by = rect.y + 40.0;
    let bh = 34.0;

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
        let bw = (label.len() as f32 * 7.2) + 12.0;
        let is_selected = app.active_tool == *tool;
        if draw_button(Rect::new(bx, by, bw, bh), label, *col, is_selected) {
            app.active_tool = *tool;
            app.pending_route_bus = None;
            app.pending_route_source = None;
        }
        bx += bw + 5.0;
    }

    // Botão de Upgrade Global rápido
    let upg_label = "★ Upgrade (10 Bytecode)";
    let upg_w = 175.0;
    if rect.x + rect.w - upg_w - 6.0 > bx {
        if draw_button(
            Rect::new(rect.x + rect.w - upg_w - 6.0, by, upg_w, bh),
            upg_label,
            Color::new(1.0, 0.75, 0.1, 1.0),
            false,
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
fn draw_mobile_header(app: &mut AppState, rect: Rect) {
    draw_panel_box(rect, " STRUCT::FORGE // MOBILE HUD ", Color::new(0.0, 0.8, 1.0, 1.0));

    let bytes = app.world.resources.get("raw_bytes").copied().unwrap_or(0);
    let shards = app.world.resources.get("crypto_shards").copied().unwrap_or(0);
    let bytecode = app.world.resources.get("bytecode").copied().unwrap_or(0);

    // Linha 1: Recursos em destaque
    draw_text(
        &format!("Bytes: {} | Shards: {} | Code: {}", bytes, shards, bytecode),
        rect.x + 8.0,
        rect.y + 28.0,
        16.0,
        Color::new(0.9, 0.95, 1.0, 1.0),
    );

    // Linha 2: Toggles Mobile
    let eco_label = if app.battery_saver { "🔋 30 FPS" } else { "⚡ 60 FPS" };
    let eco_col = if app.battery_saver { Color::new(0.3, 1.0, 0.5, 1.0) } else { Color::new(1.0, 0.8, 0.2, 1.0) };
    if draw_button(Rect::new(rect.x + 6.0, rect.y + 36.0, 92.0, 24.0), eco_label, eco_col, app.battery_saver) {
        app.battery_saver = !app.battery_saver;
    }

    let notch_label = if app.enable_safe_area { "📱 Notch: ON" } else { "📱 Notch: OFF" };
    if draw_button(Rect::new(rect.x + 104.0, rect.y + 36.0, 102.0, 24.0), notch_label, Color::new(0.5, 0.8, 1.0, 1.0), app.enable_safe_area) {
        app.enable_safe_area = !app.enable_safe_area;
    }

    // Botão Centralizar Base
    if draw_button(Rect::new(rect.x + rect.w - 86.0, rect.y + 36.0, 80.0, 24.0), "🎯 Base", Color::new(0.3, 0.9, 1.0, 1.0), false) {
        app.camera_x = 0.0;
        app.camera_y = 0.0;
    }
}

// =========================================================================
// PAINEL DE ABAS INFERIOR (PORTRAIT MOBILE)
// =========================================================================
fn draw_mobile_tabs_panel(app: &mut AppState, rect: Rect) {
    draw_panel_box(rect, "", Color::new(0.2, 0.6, 0.9, 1.0));

    // Barra de Navegação das 4 Abas
    let tab_bar_h = 36.0;
    let tab_w = (rect.w - 12.0) / 4.0;
    let tabs = [
        ("🔨 Obras", MobileTab::Build),
        ("⚡ Upgrades", MobileTab::Upgrades),
        ("📖 Academia", MobileTab::Academy),
        ("💻 Console", MobileTab::Console),
    ];

    for (i, (label, tab)) in tabs.iter().enumerate() {
        let is_active = app.active_mobile_tab == *tab;
        let tab_rect = Rect::new(rect.x + 6.0 + i as f32 * tab_w, rect.y + 4.0, tab_w - 3.0, tab_bar_h);
        let tab_col = if is_active { Color::new(0.0, 0.9, 1.0, 1.0) } else { Color::new(0.5, 0.6, 0.7, 1.0) };
        if draw_button(tab_rect, label, tab_col, is_active) {
            app.active_mobile_tab = *tab;
        }
    }

    let content_rect = Rect::new(rect.x + 6.0, rect.y + tab_bar_h + 8.0, rect.w - 12.0, rect.h - tab_bar_h - 12.0);

    match app.active_mobile_tab {
        MobileTab::Build => draw_mobile_build_tab(app, content_rect),
        MobileTab::Upgrades => draw_mobile_upgrades_tab(app, content_rect),
        MobileTab::Academy => draw_mobile_academy_tab(app, content_rect),
        MobileTab::Console => draw_mobile_console_tab(app, content_rect),
    }
}

fn draw_mobile_build_tab(app: &mut AppState, rect: Rect) {
    let tools: &[(&str, MouseTool, Color)] = &[
        ("[1] Mineradora (25B)", MouseTool::Build("byte_harvester"), Color::new(0.0, 0.85, 1.0, 1.0)),
        ("[2] Vagão Bus (20B)", MouseTool::Build("packet_bus"), Color::new(1.0, 0.85, 0.1, 1.0)),
        ("[3] Torre Laser (35B)", MouseTool::Build("firewall_node"), Color::new(1.0, 0.35, 0.35, 1.0)),
        ("[4] Conversor (40B)", MouseTool::Build("transpiler"), Color::new(0.85, 0.35, 1.0, 1.0)),
        ("[5] Escavar (+5B)", MouseTool::Excavate, Color::new(0.6, 0.8, 0.6, 1.0)),
        ("[6] Ligar &mut", MouseTool::LinkRouteRaw, Color::new(0.2, 1.0, 0.5, 1.0)),
        ("[7] Ligar Mutex", MouseTool::LinkRouteMutex, Color::new(0.3, 0.9, 1.0, 1.0)),
        ("[8] Demolir (X)", MouseTool::Demolish, Color::new(1.0, 0.25, 0.25, 1.0)),
        ("[🖐] Mover Mapa", MouseTool::PanCamera, Color::new(0.7, 0.85, 1.0, 1.0)),
    ];

    let cols = 2;
    let btn_w = (rect.w - 8.0) / cols as f32;
    let btn_h = 36.0;

    for (idx, (label, tool, col)) in tools.iter().enumerate() {
        let r = idx / cols;
        let c = idx % cols;
        let bx = rect.x + c as f32 * btn_w;
        let by = rect.y + r as f32 * (btn_h + 5.0);

        if by + btn_h > rect.y + rect.h {
            break;
        }

        let is_selected = app.active_tool == *tool;
        if draw_button(Rect::new(bx, by, btn_w - 4.0, btn_h), label, *col, is_selected) {
            app.active_tool = *tool;
            app.pending_route_bus = None;
            app.pending_route_source = None;
        }
    }
}

fn draw_mobile_upgrades_tab(app: &mut AppState, rect: Rect) {
    let cats = [
        ("packet_bus", "Vagão Packet Bus", "Velocidade + Capacidade"),
        ("byte_harvester", "Mineradora de Dados", "Taxa de Extração de Bytes"),
        ("firewall_node", "Torre Firewall Laser", "Alcance de Detecção e Dano"),
        ("transpiler", "Transpilador de Código", "Eficiência e Conversão"),
    ];

    let card_h = 42.0;
    for (i, (cat, name, desc)) in cats.iter().enumerate() {
        let cy = rect.y + i as f32 * (card_h + 6.0);
        if cy + card_h > rect.y + rect.h {
            break;
        }

        let bp = app.world.blueprints.registry.get(*cat);
        let tier_name = bp.map_or("BASIC", |b| b.tier.name());
        let col = bp.map_or(WHITE, |b| b.color);

        draw_rectangle(rect.x, cy, rect.w, card_h, Color::new(0.06, 0.08, 0.12, 0.9));
        draw_rectangle_lines(rect.x, cy, rect.w, card_h, 1.0, col);

        draw_text(name, rect.x + 8.0, cy + 18.0, 15.0, col);
        draw_text(&format!("Tier: {} | {}", tier_name, desc), rect.x + 8.0, cy + 34.0, 13.0, LIGHTGRAY);

        let upg_btn = Rect::new(rect.x + rect.w - 110.0, cy + 6.0, 102.0, 30.0);
        if draw_button(upg_btn, "impl (10 Code)", Color::new(1.0, 0.8, 0.2, 1.0), false) {
            match app.world.upgrade_global_blueprint(cat, None) {
                Ok(msg) => app.console_log = format!("impl {}: {}", cat, msg),
                Err(err) => app.console_log = err,
            }
        }
    }
}

fn draw_mobile_academy_tab(app: &mut AppState, rect: Rect) {
    let step = &TUTORIAL_STEPS[app.tutorial_idx % TUTORIAL_STEPS.len()];
    let mut y = rect.y + 16.0;

    // Título e Botão Próximo
    draw_text(step.title, rect.x + 6.0, y, 16.0, Color::new(0.2, 0.95, 1.0, 1.0));
    let next_btn = Rect::new(rect.x + rect.w - 100.0, rect.y, 94.0, 26.0);
    if draw_button(next_btn, "Próxima ▶", Color::new(0.2, 0.85, 1.0, 1.0), false) {
        app.tutorial_idx = (app.tutorial_idx + 1) % TUTORIAL_STEPS.len();
    }
    y += 20.0;

    // Significado no Rust
    draw_text("• CONCEITO EM RUST REAL:", rect.x + 6.0, y, 13.0, Color::new(1.0, 0.85, 0.2, 1.0));
    y += 15.0;
    y = draw_wrapped_text(step.real_rust_meaning, rect.x + 8.0, y, rect.w - 16.0, 13.0, WHITE);
    y += 8.0;

    // Botão Executar Exemplo Direto
    let exec_btn = Rect::new(rect.x + 6.0, (rect.y + rect.h - 36.0).max(y + 4.0), rect.w - 12.0, 32.0);
    if draw_button(exec_btn, &format!("▶ Rodar: {}", step.example_cmd), Color::new(0.2, 1.0, 0.5, 1.0), false) {
        match execute_rust_command(&mut app.world, step.example_cmd) {
            Ok(msg) => app.console_log = msg,
            Err(err) => app.console_log = err,
        }
    }
}

fn draw_mobile_console_tab(app: &mut AppState, rect: Rect) {
    draw_text(
        &format!("borrow_checker> {}", app.world.last_borrow_event),
        rect.x + 6.0,
        rect.y + 18.0,
        14.0,
        Color::new(0.4, 0.9, 1.0, 1.0),
    );

    draw_text(
        &format!("output: {}", app.console_log),
        rect.x + 6.0,
        rect.y + 38.0,
        14.0,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );

    let mut cy = rect.y + 60.0;
    draw_text("UNIDADES EM CAMPO:", rect.x + 6.0, cy, 14.0, Color::new(0.7, 0.4, 1.0, 1.0));
    cy += 18.0;

    for st in app.world.structures.iter().take(5) {
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
            rect.x + 6.0,
            cy,
            13.0,
            WHITE,
        );
        cy += 18.0;
    }
}

// =========================================================================
// VIEWPORT DO MAPA (SUPORTA TOUCH DRAG, PANNING E ZOOM)
// =========================================================================
fn draw_map_viewport(app: &mut AppState, rect: Rect) {
    draw_panel_box(
        rect,
        " MAPA DA FÁBRICA // NAVEGAÇÃO 2D ",
        Color::new(0.2, 0.65, 0.9, 1.0),
    );

    let (mx, my) = mouse_position();
    let inside_map = rect.contains(vec2(mx, my));

    // Botões de Zoom no canto do mapa (+ e - para mobile)
    let zoom_in_rect = Rect::new(rect.x + rect.w - 38.0, rect.y + 6.0, 32.0, 26.0);
    let zoom_out_rect = Rect::new(rect.x + rect.w - 74.0, rect.y + 6.0, 32.0, 26.0);

    if draw_button(zoom_in_rect, "+", Color::new(0.3, 0.9, 1.0, 1.0), false) {
        app.zoom_tile_size = (app.zoom_tile_size + 6.0).clamp(24.0, 72.0);
    }
    if draw_button(zoom_out_rect, "-", Color::new(0.3, 0.9, 1.0, 1.0), false) {
        app.zoom_tile_size = (app.zoom_tile_size - 6.0).clamp(24.0, 72.0);
    }

    // Zoom com Scroll do Mouse (quando disponível)
    if inside_map {
        let (_, wheel_y) = mouse_wheel();
        if wheel_y != 0.0 {
            app.zoom_tile_size = (app.zoom_tile_size + wheel_y * 4.0).clamp(24.0, 72.0);
        }
    }

    // Panning da Câmera (Com ferramenta Pan, Botão Direito, Botão do Meio ou Touch Drag)
    if inside_map {
        let is_pan_click = is_mouse_button_pressed(MouseButton::Right)
            || is_mouse_button_pressed(MouseButton::Middle)
            || (is_mouse_button_pressed(MouseButton::Left) && app.active_tool == MouseTool::PanCamera);

        if is_pan_click {
            app.is_panning = true;
            app.last_mouse = (mx, my);
        }
    }

    if app.is_panning {
        let is_pan_down = is_mouse_button_down(MouseButton::Right)
            || is_mouse_button_down(MouseButton::Middle)
            || (is_mouse_button_down(MouseButton::Left) && app.active_tool == MouseTool::PanCamera);

        if is_pan_down {
            let dx = mx - app.last_mouse.0;
            let dy = my - app.last_mouse.1;
            app.camera_x -= dx / app.zoom_tile_size;
            app.camera_y -= dy / app.zoom_tile_size;
            app.last_mouse = (mx, my);
        } else {
            app.is_panning = false;
        }
    }

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
                || py + ts < rect.y + 18.0
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

                    draw_line(x1, y1, x2, y2, 4.5, Color::new(line_col.r, line_col.g, line_col.b, 0.22));
                    draw_line(x1, y1, x2, y2, 2.2, line_col);
                    draw_circle(x1, y1, 2.5, line_col);
                }
                if let Some(last) = route.path.last() {
                    let lx = center_px_x + (last.0 as f32 - app.camera_x) * ts;
                    let ly = center_px_y + (last.1 as f32 - app.camera_y) * ts;
                    draw_circle(lx, ly, 2.5, line_col);
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
                    draw_line(x1, y1, x2, y2, 2.5, Color::new(1.0, 0.9, 0.2, 0.75));
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
        draw_circle_lines(px, py, pulse.radius * ts, 2.0, c);
    }

    // 5. Estruturas & Vagões (Vagões deslizam rigorosamente SOBRE a linha)
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
                    draw_rectangle_lines(cx - radius, cy - radius, radius * 2.0, radius * 2.0, 2.5, border_col);
                }
                GeometryTier::Angular => {
                    draw_poly_lines(cx, cy, 4, radius * 1.1, 45.0, 3.0, border_col);
                }
                GeometryTier::HexCore => {
                    draw_poly_lines(cx, cy, 6, radius * 1.12, 0.0, 3.0, border_col);
                    draw_poly_lines(cx, cy, 6, radius * 0.85, 30.0, 1.5, Color::new(1.0, 0.9, 0.3, 0.9));
                }
            }

            let label = if is_panic {
                "E0499"
            } else if is_waiting {
                "WAIT"
            } else {
                bp.ascii_symbol
            };
            draw_text(label, cx - radius * 0.65, cy + 5.0, ts * 0.36, border_col);

            draw_text(
                &format!("#{}", st.id),
                cx - radius + 2.0,
                cy - radius + 11.0,
                14.0,
                Color::new(0.9, 0.95, 1.0, 0.9),
            );

            let inv_sum: u32 = st.inventory.values().sum();
            let cap = bp.effective_capacity(st.custom_tier_override).max(1);
            let ratio = (inv_sum as f32 / cap as f32).clamp(0.0, 1.0);
            let bar_w = radius * 1.6;
            draw_rectangle(cx - bar_w * 0.5, cy + radius - 5.0, bar_w, 4.0, Color::new(0.15, 0.2, 0.25, 1.0));
            draw_rectangle(cx - bar_w * 0.5, cy + radius - 5.0, bar_w * ratio, 4.0, bp.color);
        }
    }

    // 6. Glitches Nativos
    for beast in &app.world.beasts {
        let bx = center_px_x + (beast.x - app.camera_x) * ts;
        let by = center_px_y + (beast.y - app.camera_y) * ts;
        if bx >= rect.x && bx <= rect.x + rect.w && by >= rect.y && by <= rect.y + rect.h {
            draw_circle(bx, by, ts * 0.35, Color::new(0.8, 0.1, 0.1, 0.85));
            draw_circle_lines(bx, by, ts * 0.38, 2.0, RED);
            draw_text("wMw", bx - 12.0, by + 4.0, 15.0, WHITE);
            let hp_ratio = (beast.hp as f32 / beast.max_hp as f32).clamp(0.0, 1.0);
            draw_rectangle(bx - 14.0, by - 18.0, 28.0 * hp_ratio, 4.0, RED);
        }
    }

    // 7. Projéteis Visíveis
    for p in &app.world.projectiles {
        let px = center_px_x + (p.x - app.camera_x) * ts;
        let py = center_px_y + (p.y - app.camera_y) * ts;
        draw_circle(px, py, 5.5, p.color);
        draw_circle(px, py, 3.0, WHITE);
    }

    // 8. Interação de Toque / Mouse: Construção, Demolição, Rotas
    if inside_map && app.active_tool != MouseTool::PanCamera {
        let h_px = center_px_x + (hovered_tx as f32 - app.camera_x) * ts - ts * 0.5;
        let h_py = center_px_y + (hovered_ty as f32 - app.camera_y) * ts - ts * 0.5;

        let is_click = is_mouse_button_pressed(MouseButton::Left);
        let is_drag = is_mouse_button_down(MouseButton::Left);

        if app.active_tool == MouseTool::Demolish {
            draw_rectangle_lines(h_px, h_py, ts, ts, 2.5, Color::new(1.0, 0.2, 0.2, 0.9));
            draw_text("X", h_px + ts * 0.38, h_py + ts * 0.62, ts * 0.45, RED);
        } else {
            draw_rectangle_lines(h_px, h_py, ts, ts, 2.0, Color::new(1.0, 1.0, 1.0, 0.65));
        }

        match app.active_tool {
            MouseTool::Build(cat) if is_click || is_drag => {
                if let Ok(id) = app.world.spawn_structure(cat, hovered_tx, hovered_ty) {
                    app.console_log = format!(
                        "Mouse Build -> executou `spawn!({}, {}, {});` (Criou #{})",
                        cat, hovered_tx, hovered_ty, id
                    );
                }
            }
            MouseTool::Excavate if is_click || is_drag => {
                if let Ok(msg) = app.world.excavate(hovered_tx, hovered_ty) {
                    app.console_log = format!(
                        "Mouse Defrag -> executou `defrag!({}, {});` ({})",
                        hovered_tx, hovered_ty, msg
                    );
                }
            }
            MouseTool::Demolish if is_click || is_drag => {
                if let Some(target_st) = app
                    .world
                    .structures
                    .iter()
                    .find(|s| s.x == hovered_tx && s.y == hovered_ty)
                {
                    let tid = target_st.id;
                    if let Ok(msg) = app.world.demolish_structure(tid) {
                        app.console_log = format!("Mouse Demolish -> executou `destroy!({});`: {}", tid, msg);
                    }
                }
            }
            MouseTool::LinkRouteRaw | MouseTool::LinkRouteMutex if is_click => {
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
                        app.console_log = format!("Vagão #{} selecionado! Agora clique na Origem (ex: Mineradora).", clicked_id);
                    } else if app.pending_route_source.is_none() {
                        app.pending_route_source = Some(clicked_id);
                        app.console_log = format!("Origem #{} selecionada! Agora clique no Destino (ex: Conversor/Base).", clicked_id);
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
                                        app.console_log = format!("Mouse Route -> executou: `{}` ({})", cmd, msg);
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

    // 9. Tooltip Informativo ao Passar Mouse
    if inside_map {
        if let Some(st) = app.world.structures.iter().find(|s| s.x == hovered_tx && s.y == hovered_ty) {
            if let Some(bp) = app.world.blueprints.registry.get(&st.category_id) {
                let inv_total: u32 = st.inventory.values().sum();
                let cap = bp.effective_capacity(st.custom_tier_override);
                let lock_txt = match st.active_mut_borrower {
                    Some(id) => format!("Emprestado (&mut #{})", id),
                    None => "Livre".to_string(),
                };

                let tip_w = 280.0_f32.min(rect.w - 16.0);
                let tip_h = 86.0;
                let tx_box = (mx + 12.0).min(rect.x + rect.w - tip_w - 6.0);
                let ty_box = (my + 12.0).min(rect.y + rect.h - tip_h - 6.0);

                let tip_border = if app.active_tool == MouseTool::Demolish { RED } else { bp.color };
                draw_rectangle(tx_box, ty_box, tip_w, tip_h, Color::new(0.03, 0.05, 0.09, 0.96));
                draw_rectangle_lines(tx_box, ty_box, tip_w, tip_h, 1.5, tip_border);

                if app.active_tool == MouseTool::Demolish {
                    draw_text(&format!("DEMOLIR #{} {}", st.id, bp.display_name), tx_box + 8.0, ty_box + 22.0, 15.0, RED);
                    draw_text(&format!("+{} Bytes, +{} Shards", bp.cost_bytes, bp.cost_shards), tx_box + 8.0, ty_box + 44.0, 13.0, WHITE);
                } else {
                    draw_text(&format!("#{} {}", st.id, bp.display_name), tx_box + 8.0, ty_box + 20.0, 15.0, bp.color);
                    draw_text(&format!("Carga: {}/{} | Status: {}", inv_total, cap, lock_txt), tx_box + 8.0, ty_box + 40.0, 13.0, WHITE);
                }
            }
        }
    }
}

// =========================================================================
// PAINEL LATERAL DIREITO (LANDSCAPE)
// =========================================================================
fn draw_right_education_panel(app: &mut AppState, rect: Rect) {
    let top_h = rect.h * 0.62;
    let bot_h = rect.h - top_h - 8.0;

    let edu_rect = Rect::new(rect.x, rect.y, rect.w, top_h);
    let db_rect = Rect::new(rect.x, rect.y + top_h + 8.0, rect.w, bot_h);

    draw_panel_box(edu_rect, " ACADEMIA RUST (EXPLICAÇÃO PASSO A PASSO) ", Color::new(0.0, 0.9, 0.7, 1.0));

    let step = &TUTORIAL_STEPS[app.tutorial_idx % TUTORIAL_STEPS.len()];
    let mut y = edu_rect.y + 28.0;

    // Título da Lição + Botão Próxima Lição
    draw_text(step.title, edu_rect.x + 12.0, y, 17.0, Color::new(0.2, 0.95, 1.0, 1.0));
    let next_btn = Rect::new(edu_rect.x + edu_rect.w - 130.0, edu_rect.y + 8.0, 120.0, 26.0);
    if draw_button(next_btn, "Próxima ▶", Color::new(0.2, 0.85, 1.0, 1.0), false) {
        app.tutorial_idx = (app.tutorial_idx + 1) % TUTORIAL_STEPS.len();
    }
    y += 24.0;

    // 1. O que é no Rust real?
    draw_text("• O QUE É NO RUST REAL?", edu_rect.x + 12.0, y, 14.0, Color::new(1.0, 0.85, 0.2, 1.0));
    y += 18.0;
    y = draw_wrapped_text(step.real_rust_meaning, edu_rect.x + 14.0, y, edu_rect.w - 28.0, 14.0, WHITE);
    y += 8.0;

    // 2. Para que serve no jogo?
    draw_text("• O QUE FAZ NA SUA FÁBRICA?", edu_rect.x + 12.0, y, 14.0, Color::new(0.3, 1.0, 0.5, 1.0));
    y += 18.0;
    y = draw_wrapped_text(step.game_effect, edu_rect.x + 14.0, y, edu_rect.w - 28.0, 14.0, LIGHTGRAY);
    y += 8.0;

    // 3. Dissecação do comando
    draw_text("• ENTENDENDO CADA PALAVRA DO COMANDO:", edu_rect.x + 12.0, y, 14.0, Color::new(0.85, 0.45, 1.0, 1.0));
    y += 18.0;
    for line in step.token_breakdown {
        draw_text(line, edu_rect.x + 14.0, y, 13.0, Color::new(0.8, 0.9, 1.0, 1.0));
        y += 16.0;
    }

    // Botão Executar Exemplo
    let exec_rect = Rect::new(edu_rect.x + 12.0, edu_rect.y + edu_rect.h - 38.0, edu_rect.w - 24.0, 30.0);
    let btn_label = format!("▶ Executar Agora: {}", step.example_cmd);
    if draw_button(exec_rect, &btn_label, Color::new(0.2, 1.0, 0.5, 1.0), false) {
        match execute_rust_command(&mut app.world, step.example_cmd) {
            Ok(msg) => app.console_log = msg,
            Err(err) => app.console_log = err,
        }
    }

    // Painel de Status das Unidades
    draw_panel_box(db_rect, " ESTRUTURAS ATIVAS & BORROW CHECKER ", Color::new(0.7, 0.4, 1.0, 1.0));
    let mut dy = db_rect.y + 26.0;

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
            db_rect.x + 10.0,
            dy,
            14.0,
            col,
        );
        dy += 18.0;
    }
}

// =========================================================================
// TERMINAL INFERIOR (LANDSCAPE)
// =========================================================================
fn draw_bottom_terminal(app: &mut AppState, rect: Rect) {
    draw_panel_box(
        rect,
        " TERMINAL RUST SIMULADO // EVENTOS & LOGS ",
        Color::new(0.2, 1.0, 0.5, 1.0),
    );

    draw_text(
        &format!("borrow_checker> {}", app.world.last_borrow_event),
        rect.x + 12.0,
        rect.y + 26.0,
        15.0,
        Color::new(0.4, 0.9, 1.0, 1.0),
    );

    draw_text(
        &format!("rust_console  > {}█", app.command_input),
        rect.x + 12.0,
        rect.y + 48.0,
        17.0,
        Color::new(0.3, 1.0, 0.4, 1.0),
    );

    draw_text(
        &format!("output        : {}", app.console_log),
        rect.x + 12.0,
        rect.y + 70.0,
        15.0,
        Color::new(1.0, 0.85, 0.2, 1.0),
    );
}

// =========================================================================
// WIDGETS AUXILIARES
// =========================================================================
fn draw_panel_box(rect: Rect, title: &str, border_color: Color) {
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, Color::new(0.05, 0.07, 0.11, 0.96));
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.5, border_color);
    if !title.is_empty() {
        draw_text(title, rect.x + 8.0, rect.y + 14.0, 14.0, border_color);
    }
}

fn draw_button(rect: Rect, text: &str, color: Color, active: bool) -> bool {
    let (mx, my) = mouse_position();
    let hovered = rect.contains(vec2(mx, my));
    let clicked = hovered && is_mouse_button_pressed(MouseButton::Left);

    let bg = if active {
        Color::new(color.r * 0.35, color.g * 0.35, color.b * 0.35, 0.95)
    } else if hovered {
        Color::new(color.r * 0.22, color.g * 0.22, color.b * 0.22, 0.85)
    } else {
        Color::new(0.06, 0.08, 0.12, 0.85)
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, bg);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, if active { 2.0 } else { 1.2 }, color);

    let font_size = 13.5;
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
