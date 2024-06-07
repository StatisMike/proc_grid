use std::fs::File;
use std::io::Write;

use grid_forge::{gen::collapse::*, map::{GridDir, GridMap2D, GridSize}, tile::{identifiable::collection::IdentTileCollection, GridPosition, TileContainer}, vis::collection::VisCollection};
use overlap::{CollapsiblePatternGrid, DebugSubscriber, OverlappingPattern};
use rand_chacha::ChaChaRng;
use utils::{GifSingleSubscriber, RngHelper, VisGridLoaderHelper, VisRotate};

mod utils;

const MAP_10X10: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/samples/seas.png");
const MAP_20X20: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/samples/roads.png");

const OUTPUTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/collapse/outputs/");

fn main() {
    // VisCollection to handle Image <-> GridMap2D roundabouts
    let mut vis_collection = VisCollection::default();

    // Load two sample maps with 90 deegrees rotation to increase variety of rules.
    let maps = VisGridLoaderHelper::new(&mut vis_collection)
        .load_w_rotate(&[MAP_10X10, MAP_20X20], &[VisRotate::None]);

    // Create overlap analyzer.
    let mut analyzer = overlap::Analyzer::<overlap::OverlappingPattern2D<2, 2>, _>::default();

    // let pattern_size = GridSize::new_xy(3, 3);
    // let pos_0 = GridPosition::new_xy(0, 0);
    // let mut map_i = 0;
    // Analyze the loaded maps
    for map in maps {
        analyzer.analyze_map(&map);

        // let collection = analyzer.get_collection();

        // for tile in grid.inner().iter_tiles() {
        //     if let overlap::PatternTileData::WithPattern { tile_type_id: _, pattern_id } = tile.as_ref() {
        //         let pattern = collection.get_tile_data(pattern_id).unwrap();
        //         let mut pattern_map = GridMap2D::new(pattern_size);
        //         for pat_pos in pattern_size.get_all_possible_positions() {
        //             pattern_map.insert_data(&pat_pos, CollapsedTileData::new(pattern.get_id_for_pos(&pos_0, &pat_pos)));
        //         }
        //         let mut pattern_buf = vis_collection.init_map_image_buffer(&pattern_size);
        //         vis_collection.draw_map(&pattern_map, &mut pattern_buf).unwrap();
        //         pattern_buf.save(format!("{OUTPUTS_DIR}patterns/{}.png" ,pattern.pattern_id())).unwrap()
        //     }
        // }
        // map_i += 1;
    }

    // let mut file = File::create("assets/enabled_by").unwrap();
    // let adjacencies = analyzer.get_adjacency();
    // for idx in adjacencies.inner().inner_ids() {
    //     for direction in GridDir::ALL_2D {
    //         writeln!(file, "------------------- Idx: {idx} is enabledin direction: {direction:?} by -------------------").unwrap();
    //         writeln!(
    //             file,
    //             "{:?}",
    //             adjacencies.inner().get_all_adjacencies_in_direction(&idx, direction).collect::<Vec<_>>())
    //         .unwrap();
    //     }
    // }

    // let mut frequencies = analyzer.get_frequency().clone();
    // frequencies.set_weight_for_pattern_id(5702316297016057130, 5);

    let outputs_size = GridSize::new_xy(30, 30);

    // Save the collapse process as a GIF
    let file = std::fs::File::create(format!("{}{}", OUTPUTS_DIR, "overlap_entrophy.gif")).unwrap();
    let subscriber = GifSingleSubscriber::new(
        file, 
        &outputs_size, 
        vis_collection.clone()
    );

    // Resolver can be reused, as it is used for the same tile type.
    let mut resolver = overlap::Resolver::default()
    .with_subscriber(Box::new(subscriber));

    // Using propagating EntrophyQueue, we will use more restrictive `identity`
    // AdjacencyRules. It will help to keep high success rate, but is a little
    // slower than PositionQueue.
    let mut rng: ChaChaRng = RngHelper::init_str("overlap_entrophy", 5).into();
    let to_collapse = CollapsiblePatternGrid::new_empty(
        outputs_size,
        analyzer.get_collection().clone(),
        analyzer.get_frequency(),
        analyzer.get_adjacency(),
    ).unwrap();

    let after_collapse = resolver
        .generate(
            to_collapse,
            &mut rng,
            &outputs_size.get_all_possible_positions(),
            EntrophyQueue::default(),
        )
        .unwrap();

    let collapsed = after_collapse.retrieve_collapsed();
    let mut out_buffer = vis_collection.init_map_image_buffer(collapsed.as_ref().size());
    vis_collection
        .draw_map(collapsed.as_ref(), &mut out_buffer)
        .unwrap();
    out_buffer
        .save(format!("{}{}", OUTPUTS_DIR, "overlap_entrophy.png"))
        .unwrap();

    // Using non-propagating PositionQueue, we will use less restrictive `border`
    // AdjacencyRules. The success rate will be still moderately high - and
    // errors can be mitigated by just retrying, as non-propagating queue is faster.

    // let to_collapse =
    //     CollapsiblePatternGrid::new_empty(outputs_size, analyzer.get_collection().clone(), analyzer.get_frequency(), analyzer.get_adjacency()).unwrap();
    
    // let mut rng: ChaChaRng = RngHelper::init_str("overlap_position",5)
    //     .into();

    // let mut retry_times = 0;
    // let mut after_collapse = None;

    // loop {
    //     RngHelper::print_state(&rng);
    //     let res = resolver
    //     .generate(
    //         to_collapse.clone(),
    //         &mut rng,
    //         &outputs_size.get_all_possible_positions(),
    //         PositionQueue::default(),
    //     );

    //     match res {
    //         Ok(map) => {
    //             after_collapse = Some(map);
    //             break;
    //         },
    //         Err(err) => {
    //             if retry_times == 50 {
    //                 panic!("too many tries: {err}");
    //             }
    //             retry_times += 1;
    //         },
    //     }
    // }
    

    // let collapsed = after_collapse.unwrap().retrieve_collapsed();
    // let mut out_buffer = vis_collection.init_map_image_buffer(collapsed.as_ref().size());
    // vis_collection
    //     .draw_map(collapsed.as_ref(), &mut out_buffer)
    //     .unwrap();
    // out_buffer
    //     .save(format!("{}{}", OUTPUTS_DIR, "overlap_position.png"))
    //     .unwrap();
}
