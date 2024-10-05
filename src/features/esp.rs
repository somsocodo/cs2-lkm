use egui::{ Color32, Pos2, FontId, FontFamily };
use once_cell::sync::Lazy;

use config::Config;
use render::Render;
use sdk::Player::Player;
use sdk::Entity::Entity;
use sdk::Icon::IconResolver;

static ICON_RESOLVER: Lazy<IconResolver> = Lazy::new(|| IconResolver::new());

pub fn draw_entity(entity: &Entity, painter: &egui::Painter){
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

pub fn draw_hitboxes(config: Config, player: &Player, ui: &egui::Ui, painter: &egui::Painter) {
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

        let mut col = config.esp_hitboxes_col_hid;
        if player.bspotted {
            col = config.esp_hitboxes_col_vis; 
        }

        let colour = Color32::from_rgba_premultiplied(col[0], col[1], col[2], col[3]);
        
        let radius_min = hitbox.min_rad_2d;
        let radius_max = hitbox.max_rad_2d;

        painter.circle_filled(bb_max_pos, radius_min, colour);
        painter.circle_filled(bb_min_pos, radius_max, colour);
        painter.line_segment([bb_min_pos, bb_max_pos], (radius_min * 2.0, colour));
    }
}

pub fn draw_bones(player: &Player, painter: &egui::Painter) {
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

pub fn draw_nametags(player: &Player, ui: &egui::Ui, painter: &egui::Painter) {
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