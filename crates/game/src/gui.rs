use core::f32;

use simulation::{Object, ObjectId};

#[derive(Default)]
pub(crate) struct Gui {}

pub(crate) struct Objects<'a> {
    pub root: &'a Object,
    pub selected: &'a Object,
}

#[derive(Default)]
pub(crate) struct Outputs {
    pub next_turn: bool,
    pub interacted: Option<ObjectId>,
    pub make_active_agent: Option<ObjectId>,
}

impl Gui {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn setup(&mut self, ctx: &egui::Context) {
        ctx.set_pixels_per_point(1.6);
    }

    pub fn tick(&mut self, ctx: &egui::Context, objects: Objects) -> Outputs {
        let mut outputs = Outputs::default();
        outputs.interacted = None;

        top_strip(ctx, objects.root, &mut outputs);
        object_ui(ctx, objects.selected, &mut outputs);

        if let Some(list) = objects.root.try_list("actions") {
            actions_ui(ctx, list, &mut outputs);
        }
        outputs
    }
}

fn top_strip(ctx: &egui::Context, obj: &Object, outputs: &mut Outputs) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            if ui.small_button("Next Turn").clicked() {
                outputs.next_turn = true;
            }
            ui.label(format!("Turn Number: {}", obj.txt("turn_number")));
            ui.separator();
            entity_button(ui, obj.child("active_agent"), 160., outputs);
        });
    });
}

fn object_ui(ctx: &egui::Context, obj: &Object, outputs: &mut Outputs) {
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
                        outputs.make_active_agent = Some(id);
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

                rows_table(ui, "people-here-grid", &rows, list, outputs, 80.);
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

                rows_table(ui, "cards-here-grid", &rows, list, outputs, 80.);
            }
        });
}

fn actions_ui(ctx: &egui::Context, list: &[Object], outputs: &mut Outputs) {
    egui::Window::new("Actions")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            let table = &[Row {
                id: "id",
                label: "Name",
                primary: "name",
                tooltip: &[],
                width: 120.,
            }];
            rows_table(ui, "actions-grid", table, list, outputs, f32::INFINITY);
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
    outputs: &mut Outputs,
    height: f32,
) {
    const IDX_WIDTH: f32 = 10.;
    const ROW_HEIGHT: f32 = 16.;

    if list.is_empty() {
        ui.label("Empty...");
        return;
    }
    egui::Grid::new(&format!("{}_heading", grid_id))
        .striped(true)
        .min_col_width(IDX_WIDTH)
        .show(ui, |ui| {
            ui.add_sized([IDX_WIDTH, ROW_HEIGHT], egui::Label::new(""));
            for row in table {
                ui.add_sized([row.width, ROW_HEIGHT], egui::Label::new(row.label));
            }
        });

    egui::ScrollArea::vertical()
        .id_salt(&format!("{}_scroll", grid_id))
        .max_height(height)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {
            if height != f32::INFINITY {
                ui.set_min_height(height);
            }
            egui::Grid::new(grid_id)
                .striped(true)
                .min_col_width(IDX_WIDTH)
                .show(ui, |ui| {
                    for (idx, obj) in list.iter().enumerate() {
                        ui.add_sized(
                            [IDX_WIDTH, ROW_HEIGHT],
                            egui::Label::new(format!("{}", idx + 1)),
                        );
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
                                    outputs.interacted = Some(obj.id(row.id));
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
    outputs: &mut Outputs,
) -> egui::Response {
    let id = obj.id("id");
    let sense = ui
        .add_enabled_ui(id.is_valid(), |ui| {
            ui.add_sized([width, 14.], egui::Button::new(obj.txt("name")).small())
        })
        .inner;
    if sense.clicked() {
        outputs.interacted = Some(id)
    }
    sense
}
