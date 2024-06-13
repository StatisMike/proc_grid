use grid_forge::{gen::collapse::*, map::GridSize, vis::collection::VisCollection};
use overlap::CollapsiblePatternGrid;
use rand_chacha::ChaChaRng;
use utils::{ArgHelper, GifSingleSubscriber, RngHelper, VisGridLoaderHelper, VisRotate};

mod utils;

const MAP: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/samples/overlap.png");

const OUTPUTS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/collapse/outputs/");

fn main() {
    let args = ArgHelper::gather();

    // VisCollection to handle Image <-> GridMap2D roundabouts
    let mut vis_collection = VisCollection::default();

    // Load two sample maps with 90 deegrees rotation to increase variety of rules.
    let maps = VisGridLoaderHelper::new(&mut vis_collection)
        .load_w_rotate(&[MAP], &[VisRotate::None, VisRotate::R90]);

    let outputs_size = GridSize::new_xy(30, 30);

    // Create overlap analyzer.
    let mut analyzer = overlap::Analyzer::<overlap::OverlappingPattern2D<3, 3>, _>::default();

    for map in maps.iter() {
        analyzer.analyze_map(map);
    }

    // Resolver can be reused, as it is used for the same tile type.
    let mut resolver = overlap::Resolver::default();

    // Save the collapse process as a GIF.
    if args.gif() {
        let file =
            std::fs::File::create(format!("{}{}", OUTPUTS_DIR, "overlap_entrophy.gif")).unwrap();

        let subscriber =
            GifSingleSubscriber::new(file, &outputs_size, vis_collection.clone()).with_rescale(3);

        resolver = resolver.with_subscriber(Box::new(subscriber));
    }

    // Using propagating EntrophyQueue, we will use more restrictive `identity`
    // AdjacencyRules. It will help to keep high success rate, but is a little
    // slower than PositionQueue.
    let mut rng: ChaChaRng = RngHelper::init_str("overlap_entrophy", 2).into();
    let to_collapse = CollapsiblePatternGrid::new_empty(
        outputs_size,
        analyzer.get_collection().clone(),
        analyzer.get_frequency(),
        analyzer.get_adjacency(),
    )
    .unwrap();

    let after_collapse = resolver
        .generate_entrophy(
            to_collapse,
            &mut rng,
            &outputs_size.get_all_possible_positions(),
        )
        .unwrap();

    let collapsed = after_collapse.retrieve_collapsed();
    let mut out_buffer = vis_collection.init_map_image_buffer(collapsed.as_ref().size());
    vis_collection
        .draw_map(collapsed.as_ref(), &mut out_buffer)
        .unwrap();
    out_buffer = image::imageops::resize(
        &out_buffer,
        outputs_size.x() * 4 * 3,
        outputs_size.y() * 4 * 3,
        image::imageops::FilterType::Nearest,
    );
    out_buffer
        .save(format!("{}{}", OUTPUTS_DIR, "overlap_entrophy.png"))
        .unwrap();

    // Using non-propagating PositionQueue, we will use less restrictive `border`
    // AdjacencyRules. The success rate will be still moderately high - and
    // errors can be mitigated by just retrying, as non-propagating queue is faster.

    let mut analyzer = overlap::Analyzer::<overlap::OverlappingPattern2D<2, 2>, _>::default();
    for map in maps.iter() {
        analyzer.analyze_map(map);
    }
    let mut resolver = overlap::Resolver::default();

    // Save the collapse process as a GIF.
    if args.gif() {
        let file =
            std::fs::File::create(format!("{}{}", OUTPUTS_DIR, "overlap_position.gif")).unwrap();

        let subscriber =
            GifSingleSubscriber::new(file, &outputs_size, vis_collection.clone()).with_rescale(3);

        resolver = resolver.with_subscriber(Box::new(subscriber));
    }

    let to_collapse = CollapsiblePatternGrid::new_empty(
        outputs_size,
        analyzer.get_collection().clone(),
        analyzer.get_frequency(),
        analyzer.get_adjacency(),
    )
    .unwrap();

    let mut rng: ChaChaRng = RngHelper::init_str("overlap_position", 0)
        .with_pos(13934)
        .into();

    let after_collapse = resolver.generate_position(
        to_collapse.clone(),
        &mut rng,
        &outputs_size.get_all_possible_positions(),
        PositionQueue::default(),
    );

    let collapsed = after_collapse.unwrap().retrieve_collapsed();
    let mut out_buffer = vis_collection.init_map_image_buffer(collapsed.as_ref().size());
    vis_collection
        .draw_map(collapsed.as_ref(), &mut out_buffer)
        .unwrap();
    out_buffer = image::imageops::resize(
        &out_buffer,
        outputs_size.x() * 4 * 3,
        outputs_size.y() * 4 * 3,
        image::imageops::FilterType::Nearest,
    );
    out_buffer
        .save(format!("{}{}", OUTPUTS_DIR, "overlap_position.png"))
        .unwrap();
}
