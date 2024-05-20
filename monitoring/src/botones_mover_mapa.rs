use egui::{Align2, RichText, Ui, Window};
use walkers::MapMemory;

fn acercar(ui: &mut Ui, map_memory: &mut MapMemory) {
    if ui
        .add_sized([40., 40.], egui::Button::new(RichText::new("➕").heading()))
        .clicked()
    {
        let _ = map_memory.zoom_in();
    }
}

fn alejar(ui: &mut Ui, map_memory: &mut MapMemory) {
    if ui
        .add_sized([40., 40.], egui::Button::new(RichText::new("➖").heading()))
        .clicked()
    {
        let _ = map_memory.zoom_out();
    }
}

pub fn zoom(ui: &Ui, map_memory: &mut MapMemory) {
    Window::new("Map")
        .collapsible(false)
        .resizable(false)
        .title_bar(false)
        .anchor(Align2::LEFT_BOTTOM, [10., -10.])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                acercar(ui, map_memory);
                alejar(ui, map_memory);
                click_boton_ir_a_inicio(ui, map_memory);
            });
        });
}

fn click_boton_ir_a_inicio(ui: &mut Ui, map_memory: &mut MapMemory) {
    if ui
        .add_sized([40., 40.], egui::Button::new(RichText::new("📍").heading()))
        .clicked()
    {
        map_memory.follow_my_position();
    }
}
