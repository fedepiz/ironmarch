use crate::object::*;
use crate::simulation::*;

#[derive(Default)]
pub struct SimView {
    pub map_lines: Vec<(V2, V2)>,
    pub map_items: Vec<MapItem>,
    pub objects: Vec<Option<Object>>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum MapItemKind {
    Site,
}

pub struct MapItem {
    pub id: ObjectId,
    pub kind: MapItemKind,
    pub name: String,
    pub image: &'static str,
    pub pos: V2,
    pub size: f32,
    pub layer: u8,
}

pub(crate) fn map_view_lines(sim: &Simulation, viewport: Extents) -> Vec<(V2, V2)> {
    let mut out = Vec::with_capacity(100);
    for (id, site) in sim.sites.iter() {
        let parent_out = !viewport.contains(site.pos);
        for neigh_id in sim.sites.greater_neighbours(id) {
            let destination = sim.sites.get(neigh_id).unwrap().pos;
            let child_out = !viewport.contains(destination);
            if !parent_out || !child_out {
                out.push((site.pos, destination));
            }
        }
    }
    out
}

pub(crate) fn map_view_items(sim: &Simulation, viewport: Extents) -> Vec<MapItem> {
    let sites = sim
        .sites
        .iter()
        .filter(|(_, site)| viewport.contains(site.pos))
        .filter_map(|(site_id, site)| {
            Some(MapItem {
                id: ObjectId(ObjectHandle::Site(site_id)),
                kind: MapItemKind::Site,
                name: String::default(),
                image: "",
                pos: site.pos,
                size: 1.,
                layer: 0,
            })
        });

    let mut items: Vec<_> = sites.collect();
    items.sort_by_key(|item| item.layer);
    items
}

pub(super) fn extract_object(sim: &mut Simulation, id: ObjectId) -> Option<Object> {
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
            obj.set("kind", "Site");
        }
    }

    Some(obj)
}
