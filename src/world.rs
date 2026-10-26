use crate::models::{
    BlueprintDatabase, GeometryTier, LogisticsRoute, NativeBeast, Projectile, RoutePhase,
    StructureEntity, VisualPulse,
};
use macroquad::prelude::Color;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileType {
    EmptyFloor,
    SolidRock,   // Bloco escuro que pode ser escavado para dar espaço e +RawBytes
    IronVein,    // Veio de RawBytes
    CrystalVein, // Veio de CryptoShards
    BeastNest,   // Ninho de Feras/Glitches
}

pub struct World {
    pub tick: u64,
    pub tick_timer: f32,
    pub modified_tiles: HashMap<(i32, i32), TileType>,
    pub structures: Vec<StructureEntity>,
    pub beasts: Vec<NativeBeast>,
    pub projectiles: Vec<Projectile>,
    pub pulses: Vec<VisualPulse>,
    pub next_entity_id: u32,
    pub resources: HashMap<String, u32>,
    pub blueprints: BlueprintDatabase,
    pub last_borrow_event: String,
}

impl World {
    pub fn new() -> Self {
        let mut resources = HashMap::new();
        resources.insert("raw_bytes".to_string(), 150);
        resources.insert("crypto_shards".to_string(), 40);
        resources.insert("bytecode".to_string(), 20);

        let mut world = Self {
            tick: 0,
            tick_timer: 0.0,
            modified_tiles: HashMap::new(),
            structures: Vec::new(),
            beasts: Vec::new(),
            projectiles: Vec::new(),
            pulses: Vec::new(),
            next_entity_id: 1,
            resources,
            blueprints: BlueprintDatabase::new(),
            last_borrow_event: "Sistema pronto. Crie rotas na grade ou use a ferramenta Demolir!".to_string(),
        };

        // Clareira inicial limpa e organizada
        for y in -6..=6 {
            for x in -9..=9 {
                world.modified_tiles.insert((x, y), TileType::EmptyFloor);
            }
        }
        world.modified_tiles.insert((-5, -2), TileType::IronVein);
        world.modified_tiles.insert((-5, -1), TileType::IronVein);
        world.modified_tiles.insert((-5, 2), TileType::CrystalVein);

        // Estruturas iniciais gratuitas para demonstração imediata
        let _ = world.spawn_structure_internal("byte_harvester", -5, -2, false); // #1
        let _ = world.spawn_structure_internal("transpiler", 3, -2, false);      // #2
        let _ = world.spawn_structure_internal("packet_bus", -5, -2, false);     // #3 (posicionado na linha!)
        let _ = world.spawn_structure_internal("packet_bus", -1, 1, false);      // #4
        let _ = world.spawn_structure_internal("firewall_node", 5, 1, false);    // #5

        // Rota automática inicial ortogonal no Vagão #3 (#1 -> #2)
        let initial_path = world.find_grid_path((-5, -2), (3, -2));
        if let Some(bus3) = world.structures.iter_mut().find(|s| s.id == 3) {
            bus3.x = -5;
            bus3.y = -2;
            bus3.render_x = -5.0;
            bus3.render_y = -2.0;
            bus3.route = Some(LogisticsRoute {
                source_id: 1,
                target_id: 2,
                phase: RoutePhase::MovingToTarget,
                use_mutex: false,
                path: initial_path,
                path_index: 0,
            });
        }

        world
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        if let Some(&tile) = self.modified_tiles.get(&(x, y)) {
            return tile;
        }
        let mut h = (x as i64)
            .wrapping_mul(374761393)
            ^ (y as i64).wrapping_mul(668265263);
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        let val = (h ^ (h >> 16)).abs() % 100;

        if val < 22 {
            TileType::SolidRock
        } else if val < 28 {
            TileType::IronVein
        } else if val < 31 {
            TileType::CrystalVein
        } else if val == 32 && (x.abs() > 10 || y.abs() > 7) {
            TileType::BeastNest
        } else {
            TileType::EmptyFloor
        }
    }

    /// Calcula caminho ortogonal estrito ao longo da grade (NUNCA diagonal)
    pub fn find_grid_path(&self, start: (i32, i32), goal: (i32, i32)) -> Vec<(i32, i32)> {
        if start == goal {
            return vec![start];
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        let mut found = false;
        let max_search = 1500;
        let mut count = 0;

        while let Some(curr) = queue.pop_front() {
            count += 1;
            if count > max_search {
                break;
            }
            if curr == goal {
                found = true;
                break;
            }

            // 4 direções estritamente ortogonais: Cima, Baixo, Esquerda, Direita
            let neighbors = [
                (curr.0 + 1, curr.1),
                (curr.0 - 1, curr.1),
                (curr.0, curr.1 + 1),
                (curr.0, curr.1 - 1),
            ];

            for next in neighbors {
                if !visited.contains(&next) {
                    let tile = self.get_tile(next.0, next.1);
                    if tile != TileType::SolidRock || next == goal {
                        visited.insert(next);
                        parent.insert(next, curr);
                        queue.push_back(next);
                    }
                }
            }
        }

        if found {
            let mut path = Vec::new();
            let mut curr = goal;
            while curr != start {
                path.push(curr);
                if let Some(&p) = parent.get(&curr) {
                    curr = p;
                } else {
                    break;
                }
            }
            path.push(start);
            path.reverse();
            path
        } else {
            // Fallback Manhattan ortogonal estrito em 'L' (respeita 100% os quadrados)
            let mut path = Vec::new();
            let mut cx = start.0;
            let mut cy = start.1;
            path.push((cx, cy));
            while cx != goal.0 {
                cx += (goal.0 - cx).signum();
                path.push((cx, cy));
            }
            while cy != goal.1 {
                cy += (goal.1 - cy).signum();
                path.push((cx, cy));
            }
            path
        }
    }

    pub fn excavate(&mut self, x: i32, y: i32) -> Result<String, String> {
        match self.get_tile(x, y) {
            TileType::SolidRock => {
                self.modified_tiles.insert((x, y), TileType::EmptyFloor);
                *self.resources.entry("raw_bytes".to_string()).or_insert(0) += 5;
                self.pulses.push(VisualPulse {
                    x: x as f32,
                    y: y as f32,
                    radius: 0.1,
                    max_radius: 0.8,
                    color: Color::new(0.4, 0.9, 1.0, 0.8),
                });
                Ok(format!(
                    "Setor ({}, {}) escavado! +5 RawBytes coletados.",
                    x, y
                ))
            }
            _ => Err(format!("O bloco ({}, {}) já está limpo.", x, y)),
        }
    }

    /// Demole uma estrutura com reembolso integral e limpeza de rotas/locks
    pub fn demolish_structure(&mut self, id: u32) -> Result<String, String> {
        let pos = self
            .structures
            .iter()
            .position(|s| s.id == id)
            .ok_or_else(|| format!("Estrutura #{} não encontrada para demolir.", id))?;

        let st = self.structures.remove(pos);

        // Reembolsa recursos gastos
        let mut refunded_bytes = 0;
        let mut refunded_shards = 0;
        if let Some(bp) = self.blueprints.registry.get(&st.category_id) {
            refunded_bytes = bp.cost_bytes;
            refunded_shards = bp.cost_shards;
            *self.resources.entry("raw_bytes".to_string()).or_insert(0) += refunded_bytes;
            *self.resources.entry("crypto_shards".to_string()).or_insert(0) += refunded_shards;
        }

        // Limpa locks e rotas associadas a este ID
        self.release_borrows_held_by(id);
        for other in &mut self.structures {
            if other.active_mut_borrower == Some(id) {
                other.active_mut_borrower = None;
            }
            if let Some(r) = &other.route {
                if r.source_id == id || r.target_id == id {
                    other.route = None;
                }
            }
        }

        self.pulses.push(VisualPulse {
            x: st.render_x,
            y: st.render_y,
            radius: 0.1,
            max_radius: 1.25,
            color: Color::new(1.0, 0.25, 0.25, 0.95),
        });

        Ok(format!(
            "Estrutura #{} ({}) desmontada! Reembolso: +{} RawBytes, +{} Shards.",
            id, st.category_id, refunded_bytes, refunded_shards
        ))
    }

    pub fn spawn_structure(&mut self, category_id: &str, x: i32, y: i32) -> Result<u32, String> {
        self.spawn_structure_internal(category_id, x, y, true)
    }

    fn spawn_structure_internal(
        &mut self,
        category_id: &str,
        x: i32,
        y: i32,
        spend_resources: bool,
    ) -> Result<u32, String> {
        let bp = self
            .blueprints
            .registry
            .get(category_id)
            .ok_or_else(|| format!("Categoria '{}' não existe.", category_id))?
            .clone();

        if self.get_tile(x, y) == TileType::SolidRock {
            return Err(format!(
                "Bloqueado por Rocha/NullSector em ({}, {})! Use a ferramenta Escavar primeiro.",
                x, y
            ));
        }

        if self.structures.iter().any(|s| s.x == x && s.y == y) {
            return Err(format!("Já existe uma estrutura em ({}, {}).", x, y));
        }

        if spend_resources {
            let cur_bytes = self.resources.get("raw_bytes").copied().unwrap_or(0);
            let cur_shards = self.resources.get("crypto_shards").copied().unwrap_or(0);
            if cur_bytes < bp.cost_bytes || cur_shards < bp.cost_shards {
                return Err(format!(
                    "Recursos insuficientes! Precisa de {} RawBytes e {} Shards.",
                    bp.cost_bytes, bp.cost_shards
                ));
            }
            *self.resources.entry("raw_bytes".to_string()).or_insert(0) -= bp.cost_bytes;
            *self.resources.entry("crypto_shards".to_string()).or_insert(0) -= bp.cost_shards;
        }

        let id = self.next_entity_id;
        self.next_entity_id += 1;

        self.pulses.push(VisualPulse {
            x: x as f32,
            y: y as f32,
            radius: 0.1,
            max_radius: 1.1,
            color: bp.color,
        });

        self.structures.push(StructureEntity {
            id,
            category_id: category_id.to_string(),
            x,
            y,
            render_x: x as f32,
            render_y: y as f32,
            hp: bp.max_hp,
            inventory: HashMap::new(),
            custom_tier_override: None,
            dir: (0, 0),
            active_mut_borrower: None,
            route: None,
            fire_cooldown: 0.0,
        });

        Ok(id)
    }

    pub fn upgrade_global_blueprint(
        &mut self,
        category_id: &str,
        target_tier: Option<GeometryTier>,
    ) -> Result<String, String> {
        let bp = self
            .blueprints
            .registry
            .get_mut(category_id)
            .ok_or_else(|| format!("Struct '{}' não encontrada.", category_id))?;

        let new_tier = target_tier.unwrap_or_else(|| bp.tier.next());
        let cost_bytecode = 10;
        let cur_bc = self.resources.get("bytecode").copied().unwrap_or(0);

        if cur_bc < cost_bytecode {
            return Err(format!(
                "Precisa de {} Bytecode para evoluir a Struct global '{}'!",
                cost_bytecode, category_id
            ));
        }

        *self.resources.entry("bytecode".to_string()).or_insert(0) -= cost_bytecode;
        bp.tier = new_tier;

        for s in &self.structures {
            if s.category_id == category_id {
                self.pulses.push(VisualPulse {
                    x: s.render_x,
                    y: s.render_y,
                    radius: 0.1,
                    max_radius: 1.3,
                    color: Color::new(0.2, 1.0, 0.5, 0.9),
                });
            }
        }

        Ok(format!(
            "impl {} {{ tier = {:?}; }} -> Todas as unidades evoluíram!",
            category_id, new_tier
        ))
    }

    pub fn release_borrows_held_by(&mut self, cart_id: u32) {
        for s in &mut self.structures {
            if s.active_mut_borrower == Some(cart_id) {
                s.active_mut_borrower = None;
            }
        }
    }

    pub fn update_continuous(&mut self, dt: f32) {
        // Interpolação contínua e suave da posição sobre a linha
        for s in &mut self.structures {
            s.render_x += (s.x as f32 - s.render_x) * (dt * 12.0).min(1.0);
            s.render_y += (s.y as f32 - s.render_y) * (dt * 12.0).min(1.0);
            if s.fire_cooldown > 0.0 {
                s.fire_cooldown -= dt;
            }
        }

        // Feras nativas
        for beast in &mut self.beasts {
            let dist = (beast.x * beast.x + beast.y * beast.y).sqrt().max(0.1);
            beast.x -= (beast.x / dist) * dt * 1.2;
            beast.y -= (beast.y / dist) * dt * 1.2;
        }

        // Torres disparam projéteis visíveis
        let mut new_projectiles = Vec::new();
        for s in &mut self.structures {
            if s.category_id == "firewall_node" && s.fire_cooldown <= 0.0 {
                if let Some(target) = self.beasts.iter().find(|b| {
                    let dx = b.x - s.render_x;
                    let dy = b.y - s.render_y;
                    (dx * dx + dy * dy).sqrt() <= 7.5
                }) {
                    s.fire_cooldown = 0.65;
                    new_projectiles.push(Projectile {
                        x: s.render_x,
                        y: s.render_y,
                        target_x: target.x,
                        target_y: target.y,
                        speed: 14.0,
                        damage: 15,
                        color: Color::new(1.0, 0.35, 0.35, 1.0),
                    });
                }
            }
        }
        self.projectiles.extend(new_projectiles);

        // Voo dos projéteis
        let mut remaining_proj = Vec::new();
        for mut p in self.projectiles.drain(..) {
            let dx = p.target_x - p.x;
            let dy = p.target_y - p.y;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist < 0.35 {
                if let Some(beast) = self.beasts.iter_mut().find(|b| {
                    ((b.x - p.target_x).powi(2) + (b.y - p.target_y).powi(2)).sqrt() < 1.2
                }) {
                    beast.hp -= p.damage;
                }
                self.pulses.push(VisualPulse {
                    x: p.target_x,
                    y: p.target_y,
                    radius: 0.1,
                    max_radius: 0.9,
                    color: p.color,
                });
            } else {
                p.x += (dx / dist) * p.speed * dt;
                p.y += (dy / dist) * p.speed * dt;
                remaining_proj.push(p);
            }
        }
        self.projectiles = remaining_proj;

        let before = self.beasts.len();
        self.beasts.retain(|b| b.hp > 0);
        let killed = before.saturating_sub(self.beasts.len());
        if killed > 0 {
            *self.resources.entry("crypto_shards".to_string()).or_insert(0) += (killed as u32) * 4;
        }

        for pulse in &mut self.pulses {
            pulse.radius += dt * 2.2;
        }
        self.pulses.retain(|p| p.radius < p.max_radius);

        self.tick_timer += dt;
        if self.tick_timer >= 0.20 {
            self.tick_timer = 0.0;
            self.update_logical_tick();
        }
    }

    fn update_logical_tick(&mut self) {
        self.tick += 1;

        // 1. Mineração
        for idx in 0..self.structures.len() {
            let (x, y, cat, override_tier) = {
                let s = &self.structures[idx];
                (s.x, s.y, s.category_id.clone(), s.custom_tier_override)
            };

            if cat == "byte_harvester" && self.tick % 2 == 0 {
                let tile = self.get_tile(x, y);
                let is_crystal = tile == TileType::CrystalVein;

                if let Some(bp) = self.blueprints.registry.get(&cat) {
                    let cap = bp.effective_capacity(override_tier);
                    let s = &mut self.structures[idx];
                    let current: u32 = s.inventory.values().sum();
                    if current < cap {
                        *s.inventory.entry("raw_bytes".to_string()).or_insert(0) += 3;
                    }
                }
                *self.resources.entry("raw_bytes".to_string()).or_insert(0) += 2;
                if is_crystal {
                    *self.resources.entry("crypto_shards".to_string()).or_insert(0) += 1;
                }
            }

            if cat == "transpiler" && self.tick % 3 == 0 {
                let s = &mut self.structures[idx];
                let held = s.inventory.get("raw_bytes").copied().unwrap_or(0);
                if held >= 4 {
                    *s.inventory.entry("raw_bytes".to_string()).or_insert(0) -= 4;
                    *s.inventory.entry("bytecode".to_string()).or_insert(0) += 2;
                    *self.resources.entry("bytecode".to_string()).or_insert(0) += 2;
                    self.pulses.push(VisualPulse {
                        x: s.render_x,
                        y: s.render_y,
                        radius: 0.1,
                        max_radius: 0.85,
                        color: Color::new(0.9, 0.3, 1.0, 0.8),
                    });
                }
            }
        }

        // 2. Logística com movimento estrito sobre a linha
        self.process_logistics_and_borrows();

        // 3. Feras nativas
        if self.tick % 30 == 0 && self.beasts.len() < 4 {
            let id = self.next_entity_id;
            self.next_entity_id += 1;
            let offset = ((self.tick as i32 % 5) - 2) as f32 * 2.5;
            self.beasts.push(NativeBeast {
                id,
                x: 11.0,
                y: offset,
                hp: 45,
                max_hp: 45,
            });
        }
    }

    /// Executa o movimento do carrinho rigorosamente SOBRE a linha ortogonal
    fn process_logistics_and_borrows(&mut self) {
        let len = self.structures.len();

        for i in 0..len {
            let Some(route) = self.structures[i].route.clone() else {
                continue;
            };

            let cart_id = self.structures[i].id;
            let cart_override = self.structures[i].custom_tier_override;
            let cart_cap = self
                .blueprints
                .registry
                .get(&self.structures[i].category_id)
                .map(|bp| bp.effective_capacity(cart_override))
                .unwrap_or(25);

            match route.phase {
                RoutePhase::BorrowPanic(_) => continue,

                RoutePhase::WaitingMutex => {
                    let current_carry: u32 = self.structures[i].inventory.values().sum();
                    let next_phase = if current_carry == 0 {
                        RoutePhase::MovingToSource
                    } else {
                        RoutePhase::MovingToTarget
                    };
                    if let Some(r) = &mut self.structures[i].route {
                        r.phase = next_phase;
                    }
                }

                RoutePhase::MovingToTarget => {
                    let path_len = route.path.len();
                    if path_len == 0 {
                        continue;
                    }

                    // Se ainda não atingiu o final da linha, avança passo a passo
                    if route.path_index + 1 < path_len {
                        let next_pos = if let Some(r) = &mut self.structures[i].route {
                            r.path_index += 1;
                            Some(r.path[r.path_index])
                        } else {
                            None
                        };
                        if let Some((nx, ny)) = next_pos {
                            self.structures[i].x = nx;
                            self.structures[i].y = ny;
                        }
                    } else {
                        // Chegou no nó de destino!
                        let dst_id = route.target_id;
                        let current_holder = self
                            .structures
                            .iter()
                            .find(|s| s.id == dst_id)
                            .and_then(|s| s.active_mut_borrower);

                        match current_holder {
                            Some(other_id) if other_id != cart_id => {
                                if route.use_mutex {
                                    if let Some(r) = &mut self.structures[i].route {
                                        r.phase = RoutePhase::WaitingMutex;
                                    }
                                    self.last_borrow_event = format!(
                                        "Mutex::lock() -> Vagão #{} aguardando vaga no destino #{}.",
                                        cart_id, dst_id
                                    );
                                } else {
                                    let err_msg = format!(
                                        "ERRO [E0499]: Colisão de &mut no destino #{} entre Vagão #{} e #{}!",
                                        dst_id, cart_id, other_id
                                    );
                                    self.last_borrow_event = err_msg.clone();
                                    if let Some(r) = &mut self.structures[i].route {
                                        r.phase = RoutePhase::BorrowPanic(err_msg);
                                    }
                                }
                            }
                            _ => {
                                if let Some(dst_st) =
                                    self.structures.iter_mut().find(|s| s.id == dst_id)
                                {
                                    dst_st.active_mut_borrower = Some(cart_id);
                                }
                                if let Some(r) = &mut self.structures[i].route {
                                    r.phase = RoutePhase::BorrowingTarget(2);
                                }
                                self.last_borrow_event = format!(
                                    "Borrow Ativo: Vagão #{} descarregando via &mut em #{}.",
                                    cart_id, dst_id
                                );
                            }
                        }
                    }
                }

                RoutePhase::BorrowingTarget(ticks_left) => {
                    let dst_id = route.target_id;
                    let carried = self.structures[i]
                        .inventory
                        .get("raw_bytes")
                        .copied()
                        .unwrap_or(0);

                    if carried > 0 {
                        *self.structures[i]
                            .inventory
                            .entry("raw_bytes".to_string())
                            .or_insert(0) = 0;
                        if let Some(dst_st) = self.structures.iter_mut().find(|s| s.id == dst_id) {
                            *dst_st
                                .inventory
                                .entry("raw_bytes".to_string())
                                .or_insert(0) += carried;
                        }
                    }

                    if ticks_left <= 1 {
                        if let Some(dst_st) = self.structures.iter_mut().find(|s| s.id == dst_id) {
                            dst_st.active_mut_borrower = None;
                        }
                        if let Some(r) = &mut self.structures[i].route {
                            r.phase = RoutePhase::MovingToSource;
                        }
                        self.last_borrow_event = format!(
                            "Drop(&mut #{}) -> Vagão #{} entregou carga e retorna à origem pela linha.",
                            dst_id, cart_id
                        );
                    } else if let Some(r) = &mut self.structures[i].route {
                        r.phase = RoutePhase::BorrowingTarget(ticks_left - 1);
                    }
                }

                RoutePhase::MovingToSource => {
                    // Recua passo a passo sobre a exata mesma linha até a origem
                    if route.path_index > 0 {
                        let prev_pos = if let Some(r) = &mut self.structures[i].route {
                            r.path_index -= 1;
                            Some(r.path[r.path_index])
                        } else {
                            None
                        };
                        if let Some((px, py)) = prev_pos {
                            self.structures[i].x = px;
                            self.structures[i].y = py;
                        }
                    } else {
                        // Chegou na origem!
                        let src_id = route.source_id;
                        let current_holder = self
                            .structures
                            .iter()
                            .find(|s| s.id == src_id)
                            .and_then(|s| s.active_mut_borrower);

                        match current_holder {
                            Some(other_id) if other_id != cart_id => {
                                if route.use_mutex {
                                    if let Some(r) = &mut self.structures[i].route {
                                        r.phase = RoutePhase::WaitingMutex;
                                    }
                                    self.last_borrow_event = format!(
                                        "Mutex::lock() -> Vagão #{} aguardando vaga na origem #{}.",
                                        cart_id, src_id
                                    );
                                } else {
                                    let err_msg = format!(
                                        "ERRO [E0499]: Vagão #{} tentou pegar &mut de #{}, mas #{} já está usando!",
                                        cart_id, src_id, other_id
                                    );
                                    self.last_borrow_event = err_msg.clone();
                                    if let Some(r) = &mut self.structures[i].route {
                                        r.phase = RoutePhase::BorrowPanic(err_msg);
                                    }
                                }
                            }
                            _ => {
                                if let Some(src_st) =
                                    self.structures.iter_mut().find(|s| s.id == src_id)
                                {
                                    src_st.active_mut_borrower = Some(cart_id);
                                }
                                if let Some(r) = &mut self.structures[i].route {
                                    r.phase = RoutePhase::BorrowingSource(3);
                                }
                                self.last_borrow_event = format!(
                                    "Borrow Ativo: Vagão #{} pegou &mut exclusivo da Mineradora #{}.",
                                    cart_id, src_id
                                );
                            }
                        }
                    }
                }

                RoutePhase::BorrowingSource(ticks_left) => {
                    let src_id = route.source_id;
                    let mut avail = 0;
                    if let Some(src_st) = self.structures.iter().find(|s| s.id == src_id) {
                        avail = src_st.inventory.get("raw_bytes").copied().unwrap_or(0);
                    }

                    let held: u32 = self.structures[i].inventory.values().sum();
                    let space = cart_cap.saturating_sub(held);
                    let transfer = avail.min(space).min(8);

                    if transfer > 0 {
                        if let Some(src_st) = self.structures.iter_mut().find(|s| s.id == src_id) {
                            *src_st
                                .inventory
                                .entry("raw_bytes".to_string())
                                .or_insert(0) -= transfer;
                        }
                        *self.structures[i]
                            .inventory
                            .entry("raw_bytes".to_string())
                            .or_insert(0) += transfer;
                    }

                    if ticks_left <= 1 {
                        if let Some(src_st) = self.structures.iter_mut().find(|s| s.id == src_id) {
                            src_st.active_mut_borrower = None;
                        }
                        if let Some(r) = &mut self.structures[i].route {
                            r.phase = RoutePhase::MovingToTarget;
                        }
                        self.last_borrow_event = format!(
                            "Drop(&mut #{}) -> Vagão #{} carregado! Deslizando pela linha para o destino.",
                            src_id, cart_id
                        );
                    } else if let Some(r) = &mut self.structures[i].route {
                        r.phase = RoutePhase::BorrowingSource(ticks_left - 1);
                    }
                }
            }
        }
    }
}
