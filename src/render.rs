use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui_window_glfw_passthrough;
use egui::{ Context, Color32, Pos2, Order, Sense, Vec2, FontId, FontDefinitions, FontFamily, FontData, Stroke};
use crossbeam::channel::Receiver;
use std::sync::Once;
use std::f32::consts::PI;
use once_cell::sync::Lazy;

use config::{SharedConfig, Config};
use sdk::Vector::Vector3;
use sdk::Player::Player;
use sdk::Entity::Entity;
use sdk::WeaponClass::{ get_grenade_class_from_index, GrenadeClass };
use sdk::Icon::IconResolver;

use crate::config::SharedActiveState;
use crate::features::grenades::GrenadeHelper;

pub fn run_overlay(
    player_receiver: Receiver<Vec<Player>>, 
    world_receiver: Receiver<Vec<Entity>>, 
    shared_keystate: SharedActiveState, 
    shared_config: SharedConfig,
    grenade_helper: GrenadeHelper) {
    
    egui_overlay::start(Render { 
        player_receiver,
        world_receiver,
        shared_activestae: shared_keystate,
        shared_config,
        config: Config::load(false),
        grenade_helper,
        grenade_name: String::new(),
        grenade_action: String::new()
    });
}

pub struct Render {
    player_receiver: Receiver<Vec<Player>>,
    world_receiver: Receiver<Vec<Entity>>,
    shared_activestae: SharedActiveState,
    shared_config: SharedConfig,
    config: Config,
    grenade_helper: GrenadeHelper,
    grenade_name: String,
    grenade_action: String
}

static ICON_RESOLVER: Lazy<IconResolver> = Lazy::new(|| IconResolver::new());


impl Render {    

    fn esp_overlay(&self, egui_context: &Context) {
        egui::Area::new("overlay")
            .interactable(false)
            .fixed_pos(Pos2::new(0.,0.))
            .order(Order::Background)
            .show(egui_context, |ui| {
                ui.allocate_at_least(Vec2::new(self.config.window_size.0 as f32, self.config.window_size.1 as f32), Sense { focusable: false, drag: false, click: false });
                let painter = ui.painter();
                if let Ok(players) = self.player_receiver.recv() {
                    for player in players.iter() {
                        if self.config.esp_hitboxes {
                            self.draw_hitboxes(player, ui, painter);
                        }
                        if self.config.esp_bones {
                            self.draw_bones(player, painter);
                        }
                        if self.config.esp_nametags {
                            self.draw_nametags(player, ui, painter);
                        }      
                    }
                }
                if let Ok(entities) = self.world_receiver.recv() {
                    for entity in entities.iter() {
                        if self.config.esp_world {
                            self.draw_entity(entity, painter);
                        }
                    }
                }
                let grenade_class: GrenadeClass = {
                    let activestate_read = self.shared_activestae.read().unwrap();
                    get_grenade_class_from_index(activestate_read.weapon_index)
                };

                let view_matrix = {
                    let activestate_read = self.shared_activestae.read().unwrap();
                    activestate_read.view_matrix
                };

                if grenade_class != GrenadeClass::Invalid {
                    for grenade in &self.grenade_helper.grenades {
                        self.grenade_helper.draw(ui, self.shared_config.clone(), &grenade, view_matrix);
                    }
                }
            });
    }

    pub fn draw_circle(
        ui: &egui::Ui, 
        pos: Vector3, 
        view_matrix: [[f32; 4]; 4],
        radius: f32,         
        fill_colour: Color32,      
        stroke_colour: Color32,
        thickness: f32,
    ) {
        let painter = ui.painter();
        
        let step = 2.0 * PI / 25.0;
        let mut points: Vec<Pos2> = Vec::new();
    
        for lat in (0..=25).map(|i| i as f32 * step) {
            let point_3d = Vector3 { x: lat.cos() * radius, y: lat.sin() * radius, z: 0.0};
            let point_2d = (pos + point_3d).world_to_screen(view_matrix);
            if point_2d.x <= 0.0 || point_2d.y <= 0.0 {
                return;
            }
            
            points.push(Pos2::new(
                point_2d.x, 
                point_2d.y));
        }
    

            painter.add(egui::Shape::convex_polygon(
                points.clone(),
                fill_colour,
                Stroke::new(thickness, stroke_colour),
            ));
        

    }
    
    pub fn text_shadow(
        painter: &egui::Painter,
        pos: Pos2,
        align: egui::Align2,
        text: &str,
        colour: Color32,
        font_id: &FontId
    ) {
        painter.text(
            Pos2::new(
                pos.x + 1.5, 
                pos.y + 1.5),
                align,
                text,
                font_id.clone(),
            Color32::BLACK
        );
        painter.text(
            pos,
            align,
            text,
            font_id.clone(),
            colour
        );
    }

    fn draw_entity(&self, entity: &Entity, painter: &egui::Painter){
        let font_id_text = FontId::new(10.0, FontFamily::Monospace);
        let font_id_icon = FontId::new(20.0, FontFamily::Name("icons".into()));

        if let Some(icon) = ICON_RESOLVER.resolve_icon(&entity.class_name.to_string()) {
            let pos = Pos2::new(
                entity.pos_2d.x, 
                entity.pos_2d.y);
            if entity.is_projectile {
                Render::text_shadow(painter, pos, egui::Align2::CENTER_BOTTOM, icon, egui::Color32::from_rgba_premultiplied(255, 100, 0, 255), &font_id_icon);
            } else if entity.is_planted_c4 {
                Render::text_shadow(painter, pos, egui::Align2::CENTER_BOTTOM, icon, egui::Color32::from_rgba_premultiplied(255, 100, 0, 255), &font_id_icon);
                if entity.ammo[0] != -1{
                    let ammo_str = format!("{}/{}", entity.ammo[0], entity.ammo[1]);
                    let mut colour = Color32::WHITE;
                    if entity.ammo[0] <= 10 {
                        colour = egui::Color32::from_rgba_premultiplied(255, 100, 0, 255);
                    }
                    if entity.ammo[0] <= 5 {
                        colour = Color32::RED;
                    }
                    Render::text_shadow(painter, pos, egui::Align2::CENTER_TOP, &ammo_str, colour, &font_id_text);
                }
            } else {
                Render::text_shadow(painter, pos, egui::Align2::CENTER_BOTTOM, icon, Color32::WHITE,&font_id_icon);
                if entity.ammo[0] != -1{
                    let ammo_str = format!("{}/{}", entity.ammo[0], entity.ammo[1]);
                    Render::text_shadow(painter, pos, egui::Align2::CENTER_TOP, &ammo_str, Color32::WHITE, &font_id_text);
                }
            }
        } else {
            painter.text(
                Pos2::new(
                    entity.pos_2d.x, 
                    entity.pos_2d.y),
                egui::Align2::CENTER_TOP,
                entity.class_name.to_string(),
                font_id_text.clone(),
                Color32::WHITE,
            );
        }

    }

    fn draw_hitboxes(&self, player: &Player, ui: &egui::Ui, painter: &egui::Painter) {
        for hitbox in player.hitboxes.iter() {            
            let bb_min = hitbox.min_bounds_2d;
            let bb_max = hitbox.max_bounds_2d;
    
            if(bb_min.x < 0.0 || bb_min.y < 0.0) && (bb_max.x < 0.0 || bb_max.y < 0.0){
                continue;
            }

            let bb_min_pos = egui::Pos2::new(bb_min.x, bb_min.y);
            let bb_max_pos = egui::Pos2::new(bb_max.x, bb_max.y);
    
            if (bb_min.x - bb_max.x).abs() > ui.available_rect_before_wrap().width() as f32 {
                continue;
            }

            let mut col = self.config.esp_hitboxes_col_hid;
            if player.bspotted {
                col = self.config.esp_hitboxes_col_vis; 
            }

            let colour = Color32::from_rgba_premultiplied(col[0], col[1], col[2], col[3]);
            
            let radius_min = hitbox.min_rad_2d;
            let radius_max = hitbox.max_rad_2d;
    
            painter.circle_filled(bb_max_pos, radius_min, colour);
            painter.circle_filled(bb_min_pos, radius_max, colour);
            painter.line_segment([bb_min_pos, bb_max_pos], (radius_min * 2.0, colour));
        }
    }

    fn draw_bones(&self, player: &Player, painter: &egui::Painter) {
        let bone_hierarchy = vec![
            (0, 1),  // Head to Neck
            (1, 2),  // Neck to Spine 3
            (2, 3),  // Spine 3 to Spine 2
            (3, 4),  // Spine 2 to Spine 1
            (4, 5),  // Spine 1 to Spine 0
            (5, 6),  // Spine 0 to Hip
    
            // Left arm
            (1, 7),  // Neck to Left Shoulder
            (7, 8),  // Left Shoulder to Left Arm
            (8, 9),  // Left Arm to Left Hand
    
            // Right arm
            (1, 10), // Neck to Right Shoulder
            (10, 11), // Right Shoulder to Right Arm
            (11, 12), // Right Arm to Right Hand
    
            // Left leg
            (6, 13), // Hip to Left Hip
            (13, 14), // Left Hip to Left Knee
            (14, 15), // Left Knee to Left Foot
    
            // Right leg
            (6, 16),  // Hip to Right Hip
            (16, 17), // Right Hip to Right Knee
            (17, 18), // Right Knee to Right Foot
        ];
    
        let color = egui::Color32::from_rgb(255, 255, 255);

        if player.hitboxes.len() > 0 {
            let head_position = player.bones_2d[0];
            let head_radius = player.hitboxes[0].max_rad_2d / 1.5;
    
            if head_position.x != -99.0 && head_position.y != -99.0 {
                painter.circle_stroke(
                    egui::Pos2::new(head_position.x, head_position.y), 
                    head_radius, 
                    (2.0, color)
                );
            }
        }
    
        for &(start_idx, end_idx) in bone_hierarchy.iter() {
            if start_idx >= player.bones_2d.len() || end_idx >= player.bones_2d.len() {
                continue;
            }
    
            let bone_start = player.bones_2d[start_idx];
            let bone_end = player.bones_2d[end_idx];
    
            if (bone_start.x == -99.0 && bone_start.y == -99.0) || 
                (bone_end.x == -99.0 && bone_end.y == -99.0) ||
                (bone_start.x == 0.0 && bone_start.y == 0.0) ||
                (bone_end.x == 0.0 && bone_end.y == 0.0) {
                continue;
            }
    
            painter.line_segment(
                [egui::Pos2::new(bone_start.x, bone_start.y), egui::Pos2::new(bone_end.x, bone_end.y)], 
                (2.0, color)
            );
        }
    }

    fn draw_nametags(&self, player: &Player, ui: &egui::Ui, painter: &egui::Painter) {
        fn format_name(name: &str) -> String {
            let trimmed_name = name.trim();
        
            if trimmed_name.len() > 7 {
                trimmed_name.chars().take(7).collect::<String>() + "..."
            } else {
                trimmed_name.to_string()
            }
        }

        let name = format_name(&player.name.to_string());
        let font_id = egui::FontId::new(13.0, FontFamily::Monospace);
        let name_layout = ui.fonts(|fonts| fonts.layout_no_wrap(name.clone(), font_id.clone(), Color32::WHITE));
        let name_size = name_layout.size();

        let health_str = player.health.to_string();

        let health_layout = ui.fonts(|fonts| fonts.layout_no_wrap(health_str.clone(), font_id.clone(), Color32::WHITE));
        let health_size = health_layout.size();

        let box_padding = 3.0;
        let box_width = name_size.x + health_size.x + 2.0 + box_padding * 2.0;
        let box_height = name_size.y + box_padding * 2.0;
    
        let box_pos = Pos2::new(
            player.pos_2d.x - box_width / 2.0, 
            player.pos_2d.y - (name_size.y / 2.0) - 13.0);

        painter.rect_filled(
            egui::Rect::from_min_size(box_pos, egui::Vec2::new(box_width, box_height)),
            1.0,
            Color32::from_rgba_unmultiplied(70, 70, 70, 150)
        );

        painter.text(
            Pos2::new(
                player.pos_2d.x - (name_size.x / 2.0) - (health_size.x / 2.0), 
                player.pos_2d.y - (name_size.y / 2.0) - 10.0),
            egui::Align2::LEFT_TOP,
            name,
            font_id.clone(),
            Color32::WHITE,
        );

        let red = (2.0 * (100.0 - player.health as f32) / 100.0).min(1.0);
        let green = (2.0 * player.health as f32 / 100.0).min(1.0);
        let color = Color32::from_rgb((red * 255.0) as u8, (green * 255.0) as u8, 0);

        painter.text(
            Pos2::new(
                player.pos_2d.x + (name_size.x / 2.0) - (health_size.x / 2.0) + 2.0,
                player.pos_2d.y - (name_size.y / 2.0) - 10.0,
                ),
            egui::Align2::LEFT_TOP,
            player.health,
            font_id.clone(),
            color,
        );
    }
}

fn setup(ctx: &Context){
    let mut fonts = FontDefinitions::default();
    let icons_font_family = FontFamily::Name("icons".into());

    fonts.font_data.insert(
        "icons".to_owned(),
        FontData::from_static(include_bytes!("./assets/undefeated.ttf"))
    );

    fonts.font_data.insert(
        "text".to_owned(),
        FontData::from_static(include_bytes!("./assets/dejavu-sans-mono.book.ttf"))
    );

    fonts
        .families
        .entry(icons_font_family.clone())
        .or_default()
        .insert(0, "icons".to_owned());

    fonts
        .families
        .entry(icons_font_family.clone())
        .or_default()
        .push("text".to_owned());

    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, "text".to_owned());

    ctx.set_fonts(fonts);
}

static ONCE: Once = Once::new();

impl EguiOverlay for Render {
    fn gui_run(
        &mut self,
        egui_context: &Context,
        _default_gfx_backend: &mut DefaultGfxBackend,
        glfw_backend: &mut egui_window_glfw_passthrough::GlfwBackend,
    ) {

        let show_gui = {
            let activestate_read = self.shared_activestae.read().unwrap();
            activestate_read.show_gui
        };

        glfw_backend.window.set_pos(0, 35); //35 for cs2 windowed
        glfw_backend.window.set_size(self.config.window_size.0, self.config.window_size.1);

        if show_gui {
            glfw_backend.set_passthrough(false);
            let mut edit_config = self.config;
            if !glfw_backend.focused {
                glfw_backend.window.focus();
            }
            egui::Window::new("modules")
            .resizable(true)
            .show(egui_context, |ui| {
                ui.checkbox(&mut edit_config.gui_visuals, "gui_visuals");
                ui.checkbox(&mut edit_config.gui_combat, "gui_combat");
                ui.checkbox(&mut edit_config.gui_grenades, "gui_grenades");
                ui.checkbox(&mut edit_config.gui_misc, "gui_misc");
            });
            if edit_config.gui_visuals{
                egui::Window::new("visuals")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.checkbox(&mut edit_config.esp_nametags, "esp_nametags");
                    ui.checkbox(&mut edit_config.esp_hitboxes, "esp_hitboxes");
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                        colour_edit_button(ui, &mut edit_config.esp_hitboxes_col_vis);
                        colour_edit_button(ui, &mut edit_config.esp_hitboxes_col_hid);
                    });
                    ui.checkbox(&mut edit_config.esp_bones, "esp_bones");
                    ui.checkbox(&mut edit_config.esp_world, "esp_world");
                });
            }
            if edit_config.gui_combat{
                egui::Window::new("combat")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.checkbox(&mut edit_config.aim_enabled, "aim_enabled");
                    ui.label("aim_fov");
                    ui.add(egui::Slider::new(&mut edit_config.aim_fov, 0.0..=360.0));
                    ui.label("aim_smoothing");
                    ui.add(egui::Slider::new(&mut edit_config.aim_smoothing, 0.0..=10.0));
                    ui.label("aim_shoot_delay");
                    ui.add(egui::Slider::new(&mut edit_config.aim_shoot_delay, 0..=500));
                    ui.checkbox(&mut edit_config.trigger_enabled, "trigger_enabled");
                });
            }
            if edit_config.gui_grenades{
                egui::Window::new("grenade helper")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.grenade_name);
                    ui.label("Action:");
                    ui.text_edit_singleline(&mut self.grenade_action);
                    if ui.button("save grenade").clicked() {
                        self.grenade_helper.save(self.grenade_name.clone(), self.grenade_action.clone());
                        self.grenade_name.clear();
                        self.grenade_action.clear();
                    }
                    if ui.button("reload").clicked() {
                        self.grenade_helper.load();
                    }
                });
            }
            if edit_config.gui_misc{
                egui::Window::new("misc")
                .resizable(true)
                .show(egui_context, |ui| {
                    ui.checkbox(&mut edit_config.ignore_team, "ignore_team");
                    if ui.button("save config").clicked() {
                        edit_config.save();
                    }
                });
            }
            if edit_config != self.config {
                self.config = edit_config;
                let mut new_config = self.shared_config.write().unwrap();
                *new_config = edit_config;
            }
        } else {
            glfw_backend.set_passthrough(true);
        }
        
        self.esp_overlay(egui_context);

        egui_context.request_repaint();

        ONCE.call_once(|| {
            setup(egui_context);
            self.grenade_helper.load();
        });
    }
}

fn colour_edit_button(ui: &mut egui::Ui, colour_array: &mut [u8; 4]) {
    let mut colour = Color32::from_rgba_premultiplied(colour_array[0], colour_array[1], colour_array[2], colour_array[3]);
    if ui.color_edit_button_srgba(&mut colour).changed() {
        *colour_array = [colour.r(), colour.g(), colour.b(), colour.a()];
    }
}