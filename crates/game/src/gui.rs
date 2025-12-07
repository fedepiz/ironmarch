use simulation::{Object, ObjectId};

#[derive(Default)]
pub(crate) struct Gui {}

#[derive(Default)]
pub(crate) struct Actions {
    pub next_turn: bool,
    pub selection: ObjectId,
    pub make_active_agent: Option<ObjectId>,
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
            ui.separator();
            entity_button(ui, obj.child("active_agent"), 160., actions);
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

                if obj.flag("can_make_active_agent") {
                    if ui.small_button("Make Active Agent").clicked() {
                        actions.make_active_agent = Some(id);
                    }
                }
            });

            if let Some(list) = obj.try_list("people_here") {
                ui.separator();
                ui.heading("People Here");
                let rows = [Row {
                    label: "Name",
                    primary: "name",
                    id: "id",
                    width: 160.,
                    ..Default::default()
                }];

                rows_table(ui, "people-here-grid", &rows, list, actions, 80.);
            }

            if let Some(list) = obj.try_list("cards_here") {
                ui.separator();
                ui.heading("Cards Here");
                let rows = [Row {
                    label: "Name",
                    primary: "name",
                    id: "id",
                    width: 160.,
                    ..Default::default()
                }];

                rows_table(ui, "cards-here-grid", &rows, list, actions, 80.);
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

#[derive(Default)]
struct Row<'a> {
    id: &'a str,
    label: &'a str,
    primary: &'a str,
    tooltip: &'a [(&'a str, &'a str)],
    width: f32,
}

fn rows_table(
    ui: &mut egui::Ui,
    grid_id: &str,
    table: &[Row],
    list: &[Object],
    actions: &mut Actions,
    height: f32,
) {
    const ROW_HEIGHT: f32 = 16.;

    if list.is_empty() {
        ui.label("Empty...");
        return;
    }
    egui::Grid::new(&format!("{}_heading", grid_id))
        .striped(true)
        .show(ui, |ui| {
            for row in table {
                ui.add_sized([row.width, ROW_HEIGHT], egui::Label::new(row.label));
            }
        });

    egui::ScrollArea::vertical()
        .id_salt(&format!("{}_scroll", grid_id))
        .max_height(height)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {
            ui.set_min_height(height);
            egui::Grid::new(grid_id).striped(true).show(ui, |ui| {
                for obj in list {
                    for row in table {
                        let primary = obj.txt(row.primary);
                        let response = if row.id.is_empty() {
                            ui.label(primary)
                        } else {
                            let sense = ui.add_sized(
                                [row.width, ROW_HEIGHT],
                                egui::Button::new(primary).small(),
                            );
                            if sense.clicked() {
                                actions.selection = obj.id(row.id);
                            }
                            sense
                        };
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
        });
}

#[inline]
fn entity_button(
    ui: &mut egui::Ui,
    obj: &Object,
    width: f32,
    actions: &mut Actions,
) -> egui::Response {
    let id = obj.id("id");
    let sense = ui
        .add_enabled_ui(id.is_valid(), |ui| {
            ui.add_sized([width, 14.], egui::Button::new(obj.txt("name")).small())
        })
        .inner;
    if sense.clicked() {
        actions.selection = id
    }
    sense
}
