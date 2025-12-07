use macroquad::prelude as mq;
use simulation::*;
use util::arena::Arena;

use crate::{assets::Assets, *};

pub fn start() {
    let config = mq::Conf {
        window_width: 1600,
        window_height: 900,
        high_dpi: true,
        ..Default::default()
    };
    macroquad::Window::from_config(config, amain());
}

async fn amain() {
    let assets = Assets::load().await.unwrap();

    let mut frame_arena = Arena::default();

    let mut sim = Simulation::new(&frame_arena);
    frame_arena.reset();

    let mut gui = gui::Gui::new();
    egui_macroquad::cfg(|ctx| gui.setup(ctx));

    let mut board = board::Board::new(20., &assets);
    let mut selected_entity = ObjectId::default();

    let mut view = simulation::SimView::default();
    // Pre-records the kind of windows the matching requested objects are

    loop {
        frame_arena.reset();
        if mq::is_key_pressed(mq::KeyCode::Escape) {
            break;
        }

        let mut request = TickRequest::default();

        let mut is_mouse_over_ui = false;
        let mut is_keyboard_taken_by_ui = false;
        egui_macroquad::ui(|ctx| {
            let actions = gui.tick(ctx, &view.root, &view.selected);
            // Request transferral
            request.end_turn = actions.next_turn;
            request.make_active = actions.make_active_agent;

            selected_entity = actions.selection;

            is_mouse_over_ui = ctx.wants_pointer_input();
            is_keyboard_taken_by_ui = ctx.wants_keyboard_input();
        });

        let map_item_ids: Vec<_> = view.map_items.iter().map(|x| x.id).collect();
        populate_board(&mut board, &view, selected_entity);

        if !is_mouse_over_ui {
            if mq::is_mouse_button_pressed(mq::MouseButton::Left) {
                selected_entity = board
                    .hovered()
                    .and_then(|handle| map_item_ids.get(handle.0))
                    .copied()
                    .unwrap_or_default();
            }
        }

        if !is_keyboard_taken_by_ui {
            update_camera_from_keyboard(&mut board);

            if mq::is_key_pressed(mq::KeyCode::Space) {
                request.end_turn = true;
            }
        }

        mq::clear_background(mq::LIGHTGRAY);
        board.draw();
        egui_macroquad::draw();

        request.view.enabled = true;

        request.view.map_viewport = {
            let convert = |v: mq::Vec2| V2::new(v.x, v.y);
            let top_left = convert(board.screen_to_world(mq::Vec2::ZERO));
            let bottom_right = convert(
                board.screen_to_world(mq::Vec2::new(mq::screen_width(), mq::screen_height())),
            );
            simulation::Extents {
                top_left,
                bottom_right,
            }
        };

        request.view.selected_object = selected_entity;

        view = sim.tick(request, &frame_arena);
        mq::next_frame().await;
    }
}

fn populate_board(board: &mut board::Board, view: &SimView, selected_entity: ObjectId) {
    board.clear();
    let mut ids = Vec::with_capacity(view.map_items.len());
    // Lines
    for (source, dest) in &view.map_lines {
        board.push_line(
            mq::Vec2::new(source.x, source.y),
            mq::Vec2::new(dest.x, dest.y),
        );
    }
    // Pawns
    for item in &view.map_items {
        let handle = board::Handle(ids.len());
        ids.push(item.id);

        let is_selected = item.id == selected_entity;

        let is_big = item.size > 1.;

        let fill_color = mq::Color::from_rgba(item.color.r, item.color.g, item.color.b, 255);
        let (border_color, text_color) = if is_selected {
            (mq::YELLOW, mq::YELLOW)
        } else {
            (mq::BLACK, mq::WHITE)
        };

        let show_name = is_selected || is_big;
        let name = if show_name { item.name.as_str() } else { "" };
        let pos = mq::Vec2::new(item.pos.x, item.pos.y);

        let font_size = if is_big { 24 } else { 18 };

        let texture = if item.image.is_empty() {
            None
        } else {
            Some(board.assets.texture(item.image))
        };

        board.push_pawn(
            handle,
            name,
            texture,
            pos,
            item.size,
            font_size,
            fill_color,
            border_color,
            text_color,
        );
    }
}

fn update_camera_from_keyboard(board: &mut board::Board) {
    let mut dtranslate = mq::Vec2::ZERO;
    let mut dzoom = 0.0;

    const TRANSLATIONS: &'static [(mq::KeyCode, (f32, f32))] = &[
        (mq::KeyCode::W, (0., -1.)),
        (mq::KeyCode::S, (0., 1.)),
        (mq::KeyCode::A, (-1., 0.)),
        (mq::KeyCode::D, (1., 0.)),
    ];
    for &(key, dv) in TRANSLATIONS {
        if !mq::is_key_down(key) {
            continue;
        }
        dtranslate += mq::Vec2::from(dv);
    }

    const ZOOM: &'static [(mq::KeyCode, f32)] = &[(mq::KeyCode::Q, 1.), (mq::KeyCode::E, -1.)];
    for &(key, dz) in ZOOM {
        if !mq::is_key_down(key) {
            continue;
        }
        dzoom += dz;
    }

    board.update_camera(dtranslate, dzoom);
}
