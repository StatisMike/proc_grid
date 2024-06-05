use grid_forge::{
    gen::collapse::{
        singular::{Analyzer, BorderAnalyzer, FrequencyHints, IdentityAnalyzer, Resolver},
        CollapsibleGrid, EntrophyQueue, PositionQueue,
    },
    map::GridSize,
    vis::collection::VisCollection,
};
use rand_chacha::ChaChaRng;
use utils::{RngHelper, VisGridLoaderHelper, VisRotate};

mod utils;

const MAP_10X10: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/samples/seas.png");
const MAP_20X20: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/samples/roads.png");

const OUTPUTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/outputs/");

fn main() {
    // VisCollection to handle Image <-> GridMap2D roundabouts
    let mut vis_collection = VisCollection::default();

    // Load two sample maps with 90 deegrees rotation to increase variety of rules.
    let maps = VisGridLoaderHelper::new(&mut vis_collection)
        .load_w_rotate(&[MAP_10X10, MAP_20X20], &[VisRotate::None, VisRotate::R90]);

    // Create Identity and Border analyzers and FrequencyRules
    let mut identity_analyzer = IdentityAnalyzer::default();
    let mut border_analyzer = BorderAnalyzer::default();
    let mut frequency_hints = FrequencyHints::default();

    // Analyze the loaded maps
    for map in maps {
        identity_analyzer.analyze(&map);
        border_analyzer.analyze(&map);
        frequency_hints.analyze_grid_map(&map);
    }

    let outputs_size = GridSize::new_xy(30, 30);

    // Resolver can be reused, as it is used for the same tile type.
    let mut resolver = Resolver::default();

    // Using propagating EntrophyQueue, we will use more restrictive `identity`
    // AdjacencyRules. It will help to keep high success rate, but is a little
    // slower than PositionQueue.
    let mut rng: ChaChaRng = RngHelper::init_str("singular_identity", 0).into();
    let mut to_collapse = CollapsibleGrid::new_empty(
        outputs_size,
        &frequency_hints,
        identity_analyzer.adjacency(),
    );
    resolver
        .generate(
            &mut to_collapse,
            &mut rng,
            &outputs_size.get_all_possible_positions(),
            EntrophyQueue::default(),
        )
        .unwrap();

    let collapsed = to_collapse.retrieve_collapsed();
    let mut out_buffer = vis_collection.init_map_image_buffer(collapsed.as_ref().size());
    vis_collection
        .draw_map(collapsed.as_ref(), &mut out_buffer)
        .unwrap();
    out_buffer
        .save(format!("{}{}", OUTPUTS_DIR, "identity_entrophy.png"))
        .unwrap();

    // Using non-propagating PositionQueue, we will use less restrictive `border`
    // AdjacencyRules. The success rate will be still moderately high - and
    // errors can be mitigated by just retrying, as non-propagating queue is faster.
    let mut rng: ChaChaRng = RngHelper::init_str("singular_border", 20)
        .with_pos(6561)
        .into();

    let mut to_collapse =
        CollapsibleGrid::new_empty(outputs_size, &frequency_hints, border_analyzer.adjacency());
    resolver
        .generate(
            &mut to_collapse,
            &mut rng,
            &outputs_size.get_all_possible_positions(),
            PositionQueue::default(),
        )
        .unwrap();

    let collapsed = to_collapse.retrieve_collapsed();
    let mut out_buffer = vis_collection.init_map_image_buffer(collapsed.as_ref().size());
    vis_collection
        .draw_map(collapsed.as_ref(), &mut out_buffer)
        .unwrap();
    out_buffer
        .save(format!("{}{}", OUTPUTS_DIR, "border_position.png"))
        .unwrap();
}
