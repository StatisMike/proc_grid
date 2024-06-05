//! Below, the example of working with [`CollapsibleResolver`] to generate a randomized GridMap2D presented.
//!
//! Generation will be done using [`CollapsibleResolver::generate`] method, which can be done only over tiles implementing
//! [`IdentifiableTile`](grid_forge::tile::identifiable::IdentifiableTile), and needs some setup to be successful:
//! - you need to provide [`FrequencyHints`], which allows the Resolver to pick the options with variable probabilities.
//! - you need to provide [`AdjacencyRules`], which allows the Resolver to choose which options are valid for given tile
//! based on its neighbours.
//! - you need to provide a *Queue*, which will decide the order in which the chosen tiles will be collapsed.
//!
//! As a source of the adjacency rules and frequency hints two sample map tiles will be used to be found in `assets/samples`.
//! To load and save images, the `"vis"` feature needs to be enabled.
//!
//! We will be mixing different rules and queues in the example.
//!
//! The goal is to generate a map of island of some kind - so the map borders needs to be compromised of water tiles,
//! which will be added at the beginning of generation.
//!
//! Afterwards, the main part of the map will be generated using more restrictive rules ([`AdjacencyIdentityAnalyzer`]), so it will
//! be done in 10x10 chunks allowing for retrying upon failure, and with less time-consuming [`PositionQueue`].
//!
//! As the last part of the generation, the uncollapsed tiles - caused either by unresolved failures or just by not being
//! taken into account during previous steps will be generated using more liberate rules ([`AdjacencyBorderAnalyzer`])
//! with more time-consuming, but less error-prone [`EntrophyQueue`].  

use std::fs::File;

use grid_forge::gen::collapse::*;
use grid_forge::map::GridDir;
use grid_forge::map::GridMap2D;
use grid_forge::map::GridSize;
use grid_forge::tile::identifiable::builders::IdentTileTraitBuilder;
use grid_forge::tile::identifiable::collection::IdentTileCollection;
use grid_forge::tile::identifiable::BasicIdentTileData;
use grid_forge::tile::GridPosition;
use grid_forge::vis::collection::VisCollection;
use grid_forge::vis::ops::*;
use grid_forge::vis::DefaultVisPixel;
use rand_chacha::ChaChaRng;
use singular::Analyzer;

use rand::SeedableRng;
use singular::CollapsibleGrid;
use singular::CollapsibleTileGrid;

// fn main() {
//     let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();
//     let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

//     let seas_img = image::open("assets/samples/seas.png")
//         .unwrap()
//         .into_rgb8();

//     let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

//     let roads_img = image::open("assets/samples/roads.png")
//         .unwrap()
//         .into_rgb8();
//     let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

//     let mut analyzer = singular::IdentityAnalyzer::default();
//     analyzer.analyze(&seas_grid);
//     analyzer.analyze(&roads_grid);

//     let mut frequency_hints = singular::FrequencyHints::default();
//     frequency_hints.analyze_grid_map(&seas_grid);
//     frequency_hints.analyze_grid_map(&roads_grid);

//     let size = GridSize::new_xy(10, 10);
//     let mut grid = CollapsibleTileGrid::new_empty(size, &frequency_hints, analyzer.adjacency());

//     // Seed for reproductability
//     let mut seed = [4u8; 32];
//     let bytes = "i am benchmarking".as_bytes();
//     seed[..bytes.len().min(32)].copy_from_slice(bytes);

//     let mut rng = ChaChaRng::from_seed(seed);

//     let mut resolver = singular::Resolver::new();
//     resolver
//         .generate(
//             &mut grid,
//             &mut rng,
//             &size.get_all_possible_positions(),
//             PositionQueue::default(),
//         )
//         .unwrap();
// }

fn main() {
    // --------------- SETUP --------------- //

    // Initialize builder, which will take care of putting new tiles on specific places.
    // As `BasicIdentifiableTile` implements `ConstructableViaIdentifierTile`, the `IdentTileTraitBuilder` can be used.
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

    // Initialize pixel collection, to retrieve pixels for each identifiable tile.
    // Tile visual information need to be provided as const generic arguments there: its `Pixel` type alongside width and height
    // of each tile as number of pixels in image buffer.
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    // Load samples as grid maps.
    let seas_img = image::open("assets/samples/seas.png")
        // let seas_img = image::open("../../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("assets/samples/roads.png")
        // let roads_img = image::open("../../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    // Construct border-based adjacency analyzer and analyze the maps.
    let mut border_analyzer = singular::BorderAnalyzer::default();
    border_analyzer.analyze(&seas_grid);
    border_analyzer.analyze(&roads_grid);

    // Construct identify-based adjacency analyzer and analyze the maps.
    let mut ident_analyzer = singular::IdentityAnalyzer::default();
    ident_analyzer.analyze(&seas_grid);
    ident_analyzer.analyze(&roads_grid);

    // Generate frequency hints on basis of provided tiles
    let mut frequency_hints = singular::FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    // Seed for reproductability.
    // let mut seed: [u8; 32] = [3; 32];
    let mut seed: [u8; 32] = [1; 32];

    for (i, byte) in "collapse_gen_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }

    let mut rng = rand_chacha::ChaChaRng::from_seed(seed);

    // let size = GridSize::new_xy(2, 1);
    // let mut collapsed_grid = GridMap2D::new(size);
    // collapsed_grid.insert_data(&GridPosition::new_xy(0, 0), CollapsedTileData::new(7698123476311124029u64));

    // Create new grid with already collapsed data.
    let size = GridSize::new_xy(50, 50);

    let mut collapsed_grid = GridMap2D::new(size);

    // We will generate the map in few parts, so we can prepare the positions arrays.
    let all_positions = size.get_all_possible_positions();

    // We will fill the collapsible grid with water tiles at borders, so we will get some islands.
    let inner = GridPosition::generate_rect_area(
        &GridPosition::new_xy(4, 4),
        &GridPosition::new_xy(45, 45),
    );
    for water_position in all_positions.iter().filter(|f| !inner.contains(f)) {
        collapsed_grid.insert_data(
            water_position,
            CollapsedTileData::new(7698123476311124029u64),
        );
    }

    let identity_positions = vec![
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(2, 2),
            &GridPosition::new_xy(14, 12),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(2, 12),
            &GridPosition::new_xy(14, 22),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(2, 28),
            &GridPosition::new_xy(14, 38),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(2, 36),
            &GridPosition::new_xy(14, 46),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(12, 2),
            &GridPosition::new_xy(22, 12),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(12, 12),
            &GridPosition::new_xy(22, 22),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(12, 28),
            &GridPosition::new_xy(22, 38),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(12, 36),
            &GridPosition::new_xy(22, 46),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(20, 2),
            &GridPosition::new_xy(30, 12),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(20, 12),
            &GridPosition::new_xy(30, 22),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(20, 28),
            &GridPosition::new_xy(30, 38),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(20, 36),
            &GridPosition::new_xy(30, 46),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(28, 2),
            &GridPosition::new_xy(38, 12),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(28, 12),
            &GridPosition::new_xy(38, 22),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(28, 28),
            &GridPosition::new_xy(38, 38),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(28, 36),
            &GridPosition::new_xy(38, 46),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(36, 2),
            &GridPosition::new_xy(46, 12),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(36, 12),
            &GridPosition::new_xy(46, 22),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(36, 28),
            &GridPosition::new_xy(46, 38),
        ),
        GridPosition::generate_rect_area(
            &GridPosition::new_xy(36, 36),
            &GridPosition::new_xy(46, 46),
        ),
    ];

    let mut grid = CollapsibleTileGrid::new_from_collapsed(
        &collapsed_grid,
        &frequency_hints,
        ident_analyzer.adjacency(),
    )
    .unwrap();

    // TESTING WTF
    use std::io::Write;
    let mut file = File::create("assets/id_idx_remap").unwrap();
    write!(file, "{:#?}", grid.option_data().inner()).unwrap();

    let mut file = File::create("assets/enablers").unwrap();
    for (idx, _) in grid.option_data().iter_weights() {
        for direction in GridDir::ALL_2D {
            writeln!(file, "------------------- Idx: {idx} enables below in direction: {direction:?} -------------------").unwrap();
            writeln!(
                file,
                "{:?}",
                grid.option_data()
                    .get_all_enabled_in_direction(idx, *direction)
            )
            .unwrap();
        }
    }

    let mut file = File::create("assets/enabled_by").unwrap();
    for (idx, _) in grid.option_data().iter_weights() {
        for direction in GridDir::ALL_2D {
            writeln!(file, "------------------- Idx: {idx} is enabledin direction: {direction:?} by -------------------").unwrap();
            writeln!(
                file,
                "{:?}",
                grid.option_data()
                    .get_all_enabled_by_in_direction(idx, *direction)
            )
            .unwrap();
        }
    }

    let mut file = File::create("assets/num_ways").unwrap();
    for (idx, dirs) in grid.option_data().get_ways_to_become_option().iter_tables() {
        writeln!(
            file,
            "------------------- Ways To Become for idx: {idx} -------------------"
        )
        .unwrap();
        writeln!(file, "{dirs:?}").unwrap()
    }

    // //

    // let identity_positions = [vec![GridPosition::new_xy(1, 0)]];

    let mut resolver = singular::Resolver::new();

    // Firstly handle all portions to be resolved using 'identity' rules.
    for (iter_identity, positions) in identity_positions.iter().enumerate() {
        grid.remove_uncollapsed();
        let mut retries = 0;
        while let Err(error) = resolver.generate(
            &mut grid,
            &mut rng,
            positions,
            // For 'identity' rules we will use entrophy-based positions queue.
            PositionQueue::new(
                PositionQueueStartingPoint::UpLeft,
                PositionQueueDirection::Columnwise,
            ), // EntrophyQueue::default()
        ) {
            if retries > 13 {
                println!("identity rules generation: cannot generate tile at pos: {:?} after {retries} retries", error.failed_pos());
                break;
            }
            grid.remove_uncollapsed();
            println!("identity rules retry: {retries} for iteration: {iter_identity} after failure: {error}");
            retries += 1;
        }
        println!("generated {iter_identity} identity tiles");
    }

    let mut grid = grid.change(&frequency_hints, border_analyzer.adjacency());

    // Handle all remaining positions.
    let empty = grid.empty_positions();

    let mut retries = 0;
    while let Err(error) = resolver.generate(
        &mut grid,
        &mut rng,
        &empty,
        // For 'border' rules we will use position-based queue with default settings: rowwise from Top-Left to Bottom-Down.
        EntrophyQueue::default(),
    ) {
        if retries > 5 {
            println!("border rules generation: cannot generate tile at pos: {:?} after {retries} retries", error.failed_pos());
            break;
        }
        println!("border rules retry: {retries} after failure: {error}");
        retries += 1;
    }

    let new_map = grid.into_ident(&builder).unwrap();

    let mut out_buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(&size);
    write_gridmap_identifiable(&mut out_buffer, &new_map, &collection).unwrap();

    out_buffer.save("examples/collapse.png").unwrap();
}
