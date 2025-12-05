use crate::entity::*;
use crate::object::*;
use crate::simulation::*;
use crate::sites::Sites;
use slotmap::Key;
use spatial::geom::*;

#[derive(Default)]
pub struct SimView {
    pub map_lines: Vec<(V2, V2)>,
    pub map_items: Vec<MapItem>,
    pub objects: Vec<Option<Object>>,
}

pub struct MapItem {
    pub id: ObjectId,
    pub name: String,
    pub color: bool,
    pub image: &'static str,
    pub pos: V2,
    pub size: f32,
    pub layer: u8,
}

pub(super) fn extract(sim: &Simulation, viewport: Extents, objects: &[ObjectId]) -> SimView {
    let mut view = SimView::default();
    view.map_items = map_view_items(sim, viewport);
    view.map_lines = map_view_lines(&sim.sites, viewport);
    view.objects = objects.iter().map(|&id| extract_object(sim, id)).collect();
    view
}

fn map_view_lines(sites: &Sites, viewport: Extents) -> Vec<(V2, V2)> {
    let mut out = Vec::with_capacity(100);
    for site in sites.graph.nodes() {
        let parent_out = !viewport.contains(site.pos);
        for neigh in sites.graph.greater_neighbours(site.id) {
            let destination = sites.graph[neigh.id].pos;
            let child_out = !viewport.contains(destination);
            if !parent_out || !child_out {
                out.push((site.pos, destination));
            }
        }
    }
    out
}

fn map_view_items(sim: &Simulation, viewport: Extents) -> Vec<MapItem> {
    let sites = sim.sites.data.values().filter_map(|site| {
        if !site.bound_entity.is_null() {
            return None;
        }
        let pos = sim.sites.pos_of(site.id);
        if !viewport.contains(pos) {
            return None;
        }
        Some(MapItem {
            id: ObjectId(ObjectHandle::Site(site.id)),
            name: String::default(),
            color: false,
            image: "",
            pos,
            size: 1.,
            layer: 0,
        })
    });

    let locations = sim.entities.values().filter_map(|entity| {
        let pos = sim.sites.pos_of(entity.bound_site);
        if !viewport.contains(pos) {
            return None;
        }
        Some(MapItem {
            id: ObjectId(ObjectHandle::Entity(entity.id)),
            name: entity.name.clone(),
            color: true,
            image: entity.sprite,
            pos,
            size: 2.,
            layer: 1,
        })
    });

    let mut items: Vec<_> = sites.chain(locations).collect();
    items.sort_by_key(|item| item.layer);
    items
}

fn extract_object(sim: &Simulation, id: ObjectId) -> Option<Object> {
    let mut obj = Object::new();
    obj.set("id", id);

    match id.0 {
        ObjectHandle::Null => {
            return None;
        }

        ObjectHandle::Global => {
            obj.set("turn_number", format!("{}", sim.turn_number));
        }

        ObjectHandle::Site(_) => {
            obj.set("name", "Site");
            obj.set("kind", "Site");
        }

        ObjectHandle::Entity(subject) => {
            let entity = &sim.entities[subject];
            obj.set("name", &entity.name);
            obj.set("kind", entity.kind_name);

            let faction = sim.entities[subject]
                .hierarchies
                .parent(HierarchyName::Faction);
            obj.set("faction", &sim.entities[faction].name);

            let root = &sim.entities[sim.entities.root_of(HierarchyName::Faction, entity.id)].name;
            obj.set("root", root);
        }
    }

    Some(obj)
}
