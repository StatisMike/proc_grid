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

use std::time::Instant;

use grid_forge::gen::collapse::*;
use grid_forge::map::GridSize;
use grid_forge::tile::identifiable::builders::IdentTileTraitBuilder;
use grid_forge::tile::identifiable::BasicIdentTileData;
use grid_forge::vis::collection::VisCollection;
use grid_forge::vis::ops::*;
use grid_forge::vis::DefaultVisPixel;

use rand::SeedableRng;
use rand_chacha::ChaChaRng;

fn main() {
    // --------------- SETUP --------------- //

    let start = Instant::now();

    // Initialize builder, which will take care of putting new tiles on specific places.
    // As `BasicIdentifiableTile` implements `ConstructableViaIdentifierTile`, the `IdentTileTraitBuilder` can be used.
    let builder = IdentTileTraitBuilder::<BasicIdentTileData>::default();

    // Initialize pixel collection, to retrieve pixels for each identifiable tile.
    // Tile visual information need to be provided as const generic arguments there: its `Pixel` type alongside width and height
    // of each tile as number of pixels in image buffer.
    let mut collection = VisCollection::<DefaultVisPixel, 4, 4>::default();

    let mut analyzer =
        overlap::Analyzer::<overlap::OverlappingPattern2D<3, 3>, BasicIdentTileData>::default();

    println!("{}, analyzing seas img", start.elapsed().as_secs_f32());

    // Load samples as grid maps.
    let seas_img = image::open("assets/samples/seas.png")
        // let seas_img = image::open("../../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    analyzer.analyze_map(&seas_grid);

    println!("{}, analyzing roads img", start.elapsed().as_secs_f32());

    let roads_img = image::open("assets/samples/roads.png")
        // let roads_img = image::open("../../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    analyzer.analyze_map(&roads_grid);

    let pattern_rules = analyzer.get_adjacency();
    let pattern_collection = analyzer.get_collection();
    let pattern_freq = analyzer.get_frequency();

    println!("{}, collapsing!", start.elapsed().as_secs_f32());

    // Seed for reproductability.
    // let mut seed: [u8; 32] = [3; 32];
    let mut seed: [u8; 32] = [0; 32];

    for (i, byte) in "overlap_gen_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }

    let mut rng = ChaChaRng::from_seed(seed);

    // Create new grid with CollapsibleResolver.
    let size = GridSize::new_xy(10, 10);
    let mut resolver = overlap::Resolver::new(size);

    let positions = size.get_all_possible_positions();

    println!("{:?}", rng.get_seed());

    let mut retries = 0;
    while let Err(error) = resolver.generate(
        &mut rng,
        &positions,
        EntrophyQueue::new(2),
        pattern_collection,
        pattern_freq,
        pattern_rules,
    ) {
        if retries > 10 {
            println!("border rules generation: cannot generate tile at pos: {:?} after {retries} retries", error.failed_pos());
            break;
        }
        println!("border rules retry: {retries} after failure: {error}");
        retries += 1;
        let cloned = rng.clone();
        let stream = cloned.get_stream();
        let word_pos = cloned.get_word_pos();
        println!("stream: {stream}, word_pos: {word_pos}");
    }

    println!("{}, writing new map", start.elapsed().as_secs_f32());

    let new_map = resolver.build_grid(&builder).unwrap();

    let mut out_buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(&size);
    write_gridmap_identifiable(&mut out_buffer, &new_map, &collection).unwrap();

    out_buffer.save("examples/overlap.png").unwrap();
}
