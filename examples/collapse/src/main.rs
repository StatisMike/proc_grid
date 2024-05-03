use grid_forge::{
    gen::{adjacency::AdjacencyAnalyzer, collapse::{frequency::FrequencyHints, queue::{EntrophyQueue, PositionQueue}, resolver::CollapsibleResolver}, ms::MSAnalyzer, wfc::{analyzer::WFCAnalyzer, resolver::WFCResolver}}, gen_grid_positions_square, map::GridSize, tile::{
        identifiable::{builder::IdentTileTraitBuilder, BasicIdentifiableTile2D},
        vis::DefaultVisPixel,
    }, vis::{
        collection::VisCollection,
        ops::{init_map_image_buffer, load_gridmap_identifiable_auto, write_gridmap_identifiable},
    }
};
use rand::SeedableRng;

fn main() {
    // Initialize builder, which will take care of putting new tiles on specific places.
    // As `BasicIdentifiableTile` implements `ConstructableViaIdentifierTile`, the `IdentTileTraitBuilder` can be used.
    let builder = IdentTileTraitBuilder::<BasicIdentifiableTile2D>::default();

    // Initialize pixel collection, to retrieve pixels for each identifiable tile.
    // Tile visual information need to be provided as const generic arguments there: its `Pixel` type, width and height
    // of each tile as number of pixels in source image.
    let mut collection = VisCollection::<BasicIdentifiableTile2D, DefaultVisPixel, 4, 4>::default();

    // let paths = std::fs::read_dir("../../assets/samples").unwrap();

    // for path in paths {
    //     println!("Name: {}", path.unwrap().path().display())
    // }

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
    let mut border_analyzer = MSAnalyzer::default();
    border_analyzer.analyze(&seas_grid);
    border_analyzer.analyze(&roads_grid);

    // Construct identify-based adjacency analyzer and analyze the maps.
    let mut ident_analyzer = WFCAnalyzer::default();
    ident_analyzer.analyze(&seas_grid);
    ident_analyzer.analyze(&roads_grid);

    // Generate frequency hints on basis of provided tiles
    let mut frequency_hints = FrequencyHints::default();
    frequency_hints.analyze_grid_map(&seas_grid);
    frequency_hints.analyze_grid_map(&roads_grid);

    // Seed for reproductability.
    let mut seed: [u8; 32] = [4; 32];

    for (i, byte) in "collapse_gen_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }

    let mut rng = rand_chacha::ChaChaRng::from_seed(seed);

    // Create new grid with CollapsibleResolver.
    let size = GridSize::new(50, 50);
    let mut resolver = CollapsibleResolver::new(size);

    // We will generate the map in few parts, so we can prepare the positions arrays.
    let all_positions = size.get_all_possible_positions();

    // We will fill the collapsible grid with water tiles at borders, so we will get some islands.
    let inner = gen_grid_positions_square((4, 4), (45, 45));
    let water_tiles = all_positions
    .iter()
    .filter(|f| !inner.contains(f))
    .copied()
    .collect::<Vec<_>>();

    // resolver.fill_with_collapsed(7698123476311124029u64, &water_tiles);

    let identity_positions = vec![
        gen_grid_positions_square((2, 2), (14, 12)),
        gen_grid_positions_square((2, 12), (14, 22)),
        gen_grid_positions_square((2, 20), (14, 30)),
        gen_grid_positions_square((2, 28), (14, 38)),
        gen_grid_positions_square((2, 36), (14, 46)),

        gen_grid_positions_square((12, 2), (22, 12)),
        gen_grid_positions_square((12, 12), (22, 22)),
        gen_grid_positions_square((12, 20), (22, 30)),
        gen_grid_positions_square((12, 28), (22, 38)),
        gen_grid_positions_square((12, 36), (22, 46)),

        gen_grid_positions_square((20, 2), (30, 12)),
        gen_grid_positions_square((20, 12), (30, 22)),
        gen_grid_positions_square((20, 20), (30, 30)),
        gen_grid_positions_square((20, 28), (30, 38)),
        gen_grid_positions_square((20, 36), (30, 46)),

        gen_grid_positions_square((28, 2), (38, 12)),
        gen_grid_positions_square((28, 12), (38, 22)),
        gen_grid_positions_square((28, 20), (38, 30)),
        gen_grid_positions_square((28, 28), (38, 38)),
        gen_grid_positions_square((28, 36), (38, 46)),

        gen_grid_positions_square((36, 2), (46, 12)),
        gen_grid_positions_square((36, 12), (46, 22)),
        gen_grid_positions_square((36, 20), (46, 30)),
        gen_grid_positions_square((36, 28), (46, 38)),
        gen_grid_positions_square((36, 36), (46, 46)),
    ];

    let border_positions = all_positions
        .iter()
        .filter(|p| !identity_positions.iter().any(|v| v.contains(p)))
        .copied()
        .collect::<Vec<_>>();

    println!("border positions: {border_positions:?}");

    // Firstly handle all portions to be resolved using 'identity' rules.
    for (iter_identity, positions) in identity_positions.iter().enumerate() {
        let mut retries = 0;
        while let Err(error) = resolver.generate(
            &mut rng, 
            positions, 
            // For 'identity' rules we will use entrophy-based positions queue.
            EntrophyQueue::default(), 
            &frequency_hints, 
            ident_analyzer.adjacency()
        ) {
            if retries > 10 {
                println!("identity rules generation: cannot generate tile at pos: {:?} after {retries} retries", error.failed_pos());
                break;
            }
            println!("identity rules retry: {retries} for iteration: {iter_identity} after failure: {error}");
            retries += 1;
        }
        println!("generated {iter_identity} identity tiles");
    }

    // Handle all remaining positions.
    let uncollapsed = resolver.all_empty_positions();

    let mut retries = 0;
    while let Err(error) = resolver.generate(
        &mut rng, 
        &uncollapsed, 
        // For 'border' rules we will use position-based queue with default settings: rowwise from Top-Left to Bottom-Down.
        PositionQueue::default(), 
        &frequency_hints, 
        border_analyzer.adjacency())
        {
            if retries > 5 {
                println!("border rules generation: cannot generate tile at pos: {:?} after {retries} retries", error.failed_pos());
                break;
            }
            println!("border rules retry: {retries} after failure: {error}");
            retries += 1;
    }

    let new_map = resolver.build_grid(&builder).unwrap();

    let mut out_buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(&size);
    write_gridmap_identifiable(&mut out_buffer, &new_map, &collection).unwrap();

    out_buffer.save("collapsibleunfinished.png").unwrap();
}
