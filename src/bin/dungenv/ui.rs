use crate::{Generator, Request};
use crate::{CONTROLS, MAX_MAP_DIMENSIONS, MAX_ROOM_COUNT};
use dungen::room::RoomGraph;
use dungen::vec::vec2u;
use dungen::Configuration;

pub enum DrawOption {
    Grid,
    Corridors,
    Triangulation,
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ui(
    ui: &mut imgui::Ui,
    configuration: &mut Configuration,
    reintroduced_corridor_density: &mut f32,
    dimensions_changed: &mut bool,
    draw_option: &mut DrawOption,
    grid_width: &mut usize,
    grid_height: &mut usize,
    target_room_count: &mut usize,
    generating: &mut bool,
    generator: &Generator,
    triangulation: &RoomGraph
) {
    // ============================== User Interface
    ui.window("Configuration")
        .size([600.0, 500.0], imgui::Condition::Always)
        .position([20.0, 20.0], imgui::Condition::Always)
        .resizable(false)
        .movable(false)
        .build(|| {
            { // ============================== min_room_dimension
                ui.slider(
                    "Min Room Dimensions",
                    5,
                    100,
                    &mut configuration.min_room_dimension,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Minimum tile length of a room. Valid for both width and height.",
                    );
                }
            } // ============================== min_room_dimension


            { // ============================== max_room_dimension
                configuration.max_room_dimension = configuration
                    .max_room_dimension
                    .max(configuration.min_room_dimension);
                ui.slider(
                    "Max Room Dimensions",
                    configuration.min_room_dimension,
                    100,
                    &mut configuration.max_room_dimension,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Maximum tile length of a room. Must be greater than or equal to the minimum."
                    );
                }
            } // ============================== max_room_dimension


            { // ============================== min_padding
                ui.slider("Min padding", 3, 20, &mut configuration.min_padding);
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "The minimum distance between rooms and the map border to guarantee that doorways \
                         are accessible.");
                }
            } // ============================== min_padding


            { // ============================== doorway_offset
                configuration.doorway_offset = configuration
                    .doorway_offset
                    .min(configuration.min_room_dimension >> 1);
                ui.slider(
                    "Doorway offset",
                    1,
                    configuration.min_room_dimension >> 1,
                    &mut configuration.doorway_offset,
                );
                if ui.is_item_hovered() {
                    ui.tooltip_text("Offset from the edges of the room. Aesthetic option.");
                }
            } // ============================== doorway_offset


            { // ============================== max_fail_count
                ui.slider("Max Fail Count", 1, 200, &mut configuration.max_fail_count);
                if ui.slider(
                    "Corridor Density",
                    0.0,
                    1.0,
                    reintroduced_corridor_density,
                ) {
                    configuration.reintroduced_corridor_density =
                        ((*reintroduced_corridor_density * 1000.0) as usize, 1000);
                }
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "What percentage of edges from the triangulation on average \
                         should be reintroduced as corridors.");
                }
            } // ============================== max_fail_count

            ui.spacing();

            { // ============================== corridor costs
                ui.slider("Corridor Cost", 1, 40, &mut configuration.corridor_cost);
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Cost for the A* algorithm when we go through an already \
                         placed corridor. The relationship between this value and \
                         the other two costs determines the shape of the corridors.");
                }
                ui.slider("Straight Cost", 1, 40, &mut configuration.straight_cost);
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Cost for the A* algorithm when we go to a tile which is in the same \
                         direction (horizontal or vertical) from which we came to the current \
                         tile. When lower than the standard cost makes the corridors straight \
                         hence the name.");
                }
                ui.slider("Standard Cost", 1, 40, &mut configuration.standard_cost);
                if ui.is_item_hovered() {
                    ui.tooltip_text(
                        "Default cost for the A* algorithm. The corridors can move \
                         only horizontally or vertically.");
                }
            } // ============================== corridor costs

            ui.spacing();

            { // ============================== grid dimensions
                *dimensions_changed |= ui.slider(
                    "Grid Width",
                    configuration.min_room_dimension + configuration.min_padding * 2,
                    MAX_MAP_DIMENSIONS,
                    grid_width,
                );
                *dimensions_changed |= ui.slider(
                    "Grid Height",
                    configuration.min_room_dimension + configuration.min_padding * 2,
                    MAX_MAP_DIMENSIONS,
                    grid_height,
                );
            } // ============================== grid dimensions

            let max_room_count = (*grid_height * *grid_width).min(MAX_ROOM_COUNT);
            *target_room_count = (*target_room_count).min(max_room_count);
            ui.slider(
                "Target Room Count",
                1,
                max_room_count,
                target_room_count,
            );

            ui.label_text("Controls", CONTROLS);

            let token = ui.begin_disabled(*generating);
            if ui.button("Regenerate") {
                *generating = true;
                if generator.requests.send(Request::New {
                    configuration: configuration.clone(),
                    grid_dimensions: vec2u(*grid_width, *grid_height),
                    target_room_count: *target_room_count,
                }).is_err() {
                    return;
                };
            }

            if ui.button("Regenerate Corridors") {
                *generating = true;
                if generator.requests.send(Request::Corridors {
                    configuration: configuration.clone(),
                    grid_dimensions: vec2u(*grid_width, *grid_height),
                    triangulation: triangulation.clone()
                }).is_err() {
                    return;
                };
            }
            token.end();

            use DrawOption::*;
            ui.columns(3, "Views", false);
            if ui.button("Grid") {
                *draw_option = Grid;
            }
            ui.next_column();
            if ui.button("Corridors") {
                *draw_option = Corridors;
            }
            ui.next_column();
            if ui.button("Triangulation") {
                *draw_option = Triangulation;
            }
            ui.next_column();
            ui.columns(1, "", false);
        });
    // ============================== User Interface
}
