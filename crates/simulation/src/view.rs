use slotmap::Key;
use spatial::geom::*;
use std::borrow::Borrow;
use util::arena::Arena;

use crate::entities;
use crate::entities::*;
use crate::object::*;
use crate::simulation::*;
use crate::sites::Sites;

#[derive(Default)]
pub struct SimView {
    pub map_lines: Vec<(V2, V2)>,
    pub map_items: Vec<MapItem>,
    pub root: Object,
    pub selected: Object,
}

#[derive(Clone, Copy, Default)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct MapItem {
    pub id: ObjectId,
    pub name: String,
    pub color: RGB,
    pub image: &'static str,
    pub pos: V2,
    pub size: f32,
    pub layer: u8,
}

pub(super) fn extract(
    sim: &Simulation,
    arena: &Arena,
    viewport: Extents,
    selected: ObjectId,
) -> SimView {
    let mut view = SimView::default();
    view.map_items = map_view_items(sim, viewport);
    view.map_lines = map_view_lines(&sim.sites, viewport);

    view.root = extract_object(sim, arena, ObjectId::global());
    view.selected = extract_object(sim, arena, selected);

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
            color: RGB {
                r: 130,
                g: 130,
                b: 130,
            },
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
            color: entity.color,
            image: entity.sprite,
            pos,
            size: entity.size,
            layer: 1,
        })
    });

    let mut items: Vec<_> = sites.chain(locations).collect();
    items.sort_by_key(|item| item.layer);
    items
}

fn extract_object(sim: &Simulation, arena: &Arena, id: ObjectId) -> Object {
    match id.0 {
        ObjectHandle::Null => {
            let mut obj = Object::new();
            obj.set("id", id);
            obj
        }

        ObjectHandle::Global => {
            let mut obj = Object::new();
            obj.set("id", id);
            obj.set("turn_number", format!("{}", sim.turn_number));
            obj
        }

        ObjectHandle::Site(_) => {
            let mut obj = Object::new();
            obj.set("id", id);
            obj.set("name", "Site");
            obj.set("kind", "Site");
            obj
        }

        ObjectHandle::Entity(subject) => {
            let subject = &sim.entities[subject];
            extract_entity(sim, arena, subject)
        }
    }
}

fn extract_entity(sim: &Simulation, arena: &Arena, subject: &EntityData) -> Object {
    let mut obj = Object::new();
    obj.set("id", ObjectId::entity(subject.id));
    obj.set("name", &subject.name);
    obj.set("kind", subject.kind_name);

    let faction = subject.hierarchies.parent(HierarchyName::Faction);
    obj.set("faction", &sim.entities[faction].name);

    let root = &sim.entities[sim.entities.root_of(HierarchyName::Faction, subject.id)].name;
    obj.set("reign", root);

    obj.set(
        "hierarchy",
        extract_reference_list_from_ids(
            sim,
            sim.entities
                .ancestry(arena, HierarchyName::Faction, subject.id),
        ),
    );

    if subject.flags.get(Flag::IsPlace) {
        obj.set("people_here", {
            let list = sim.entities.children_with_flags(
                subject,
                HierarchyName::PlaceOf,
                &[entities::Flag::IsPerson],
            );
            extract_reference_list(list)
        });
    }

    obj
}

#[inline]
fn extract_reference_list_from_ids<'a, T: Borrow<EntityId>>(
    sim: &Simulation,
    iter: impl IntoIterator<Item = T>,
) -> Vec<Object> {
    extract_reference_list(iter.into_iter().map(|id| &sim.entities[*id.borrow()]))
}

#[inline]
fn extract_reference_list<'a>(iter: impl IntoIterator<Item = &'a EntityData>) -> Vec<Object> {
    iter.into_iter()
        .map(|entity| {
            let mut obj = Object::new();
            obj.set("id", ObjectId::entity(entity.id));
            obj.set("name", &entity.name);
            obj
        })
        .collect()
}
