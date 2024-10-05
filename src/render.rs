use egui_overlay::EguiOverlay;
use egui_render_three_d::ThreeDBackend as DefaultGfxBackend;
use egui_overlay::egui_window_glfw_passthrough;
use egui::{ Context, Color32, Pos2, Order, Sense, Vec2, FontId, FontDefinitions, FontFamily, FontData, Stroke};
use crossbeam::channel::Receiver;
use std::sync::Once;
use std::f32::consts::PI;

use config::{SharedConfig, Config};
use sdk::Vector::Vector3;
use sdk::Player::Player;
use sdk::Entity::Entity;
use sdk::WeaponClass::{ get_grenade_class_from_index, GrenadeClass };

use crate::config::SharedActiveState;
use crate::features::grenades::GrenadeHelper;
use crate::features::esp::{draw_entity, draw_hitboxes, draw_bones, draw_nametags};

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
                            draw_hitboxes(self.config.clone(), player, ui, painter);
                        }
                        if self.config.esp_bones {
                            draw_bones(player, painter);
                        }
                        if self.config.esp_nametags {
                            draw_nametags(player, ui, painter);
                        }      
                    }
                }
                if let Ok(entities) = self.world_receiver.recv() {
                    for entity in entities.iter() {
                        if self.config.esp_world {
                            draw_entity(entity, painter);
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