use simulation::{Object, ObjectId};

#[derive(Default)]
pub(crate) struct Gui {}

#[derive(Default)]
pub(crate) struct Actions {
    pub next_turn: bool,
    pub selection: ObjectId,
}

impl Gui {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn setup(&mut self, ctx: &egui::Context) {
        ctx.set_pixels_per_point(1.6);
    }

    pub fn tick(&mut self, ctx: &egui::Context, root: &Object, selected: &Object) -> Actions {
        let mut actions = Actions::default();
        actions.selection = selected.id("id");

        top_strip(ctx, root, &mut actions);

        object_ui(ctx, selected, &mut actions);
        actions
    }
}

fn top_strip(ctx: &egui::Context, obj: &Object, actions: &mut Actions) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            if ui.small_button("Next Turn").clicked() {
                actions.next_turn = true;
            }
            ui.label(format!("Turn Number: {}", obj.txt("turn_number")));
        });
    });
}

fn object_ui(ctx: &egui::Context, obj: &Object, actions: &mut Actions) {
    let id = obj.id("id");
    if !id.is_valid() {
        return;
    }
    egui::Window::new("Selected Entity")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.set_min_width(250.);

            ui.vertical(|ui| {
                ui.heading("Overview");
                let table = [
                    ("Name", "name"),
                    ("Kind", "kind"),
                    ("Faction", "faction"),
                    ("Reign", "reign"),
                ];
                field_table(ui, "overview-table", &table, obj);
            });

            if let Some(list) = obj.try_list("people_here") {
                ui.heading("People Here");
                if list.is_empty() {
                    ui.label("...");
                } else {
                    egui::Grid::new("people-here-grid").show(ui, |ui| {
                        for row in list {
                            let btn = egui::Button::new(row.txt("name")).small();
                            if ui.add_sized([160., 20.], btn).clicked() {
                                actions.selection = row.id("id");
                            }
                        }
                    });
                }
            }
        });
}

fn field_table(ui: &mut egui::Ui, grid_id: &str, table: &[(&str, &str)], obj: &Object) {
    egui::Grid::new(grid_id).show(ui, |ui| {
        for &(label, field) in table {
            if let Some(txt) = obj.try_text(field) {
                ui.label(label);
                ui.label(txt);
                ui.end_row();
            }
        }
    });
}

struct Row<'a> {
    label: &'a str,
    primary: &'a str,
    tooltip: &'a [(&'a str, &'a str)],
}

fn rows_table(ui: &mut egui::Ui, grid_id: &str, table: &[Row], list: &[Object]) {
    egui::Grid::new(grid_id).striped(true).show(ui, |ui| {
        if list.is_empty() {
            ui.label("Empty...");
            return;
        }
        for row in table {
            ui.label(row.label);
        }
        ui.end_row();
        for obj in list {
            for row in table {
                let primary = obj.txt(row.primary);
                let response = ui.label(primary);
                if !row.tooltip.is_empty() {
                    response.on_hover_ui(|ui| {
                        ui.heading(format!("{} {}", row.label, primary));
                        ui.separator();
                        field_table(ui, "hover-grid", row.tooltip, obj);
                    });
                }
            }
            ui.end_row();
        }
    });
}
