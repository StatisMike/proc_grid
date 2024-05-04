use grid_forge::{
    gen::{
        adjacency::AdjacencyAnalyzer,
        wfc::{analyzer::WFCAnalyzer, resolver::WFCResolver},
    },
    map::GridSize,
    tile::{
        identifiable::{builder::IdentTileTraitBuilder, BasicIdentifiableTile2D},
        vis::DefaultVisPixel,
    },
    vis::{
        collection::VisCollection,
        ops::{init_map_image_buffer, load_gridmap_identifiable_auto, write_gridmap_identifiable},
    },
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

    // Load samples as grid maps.
    let seas_img = image::open("assets/samples/seas.png")
        // let seas_img = image::open("../assets/samples/seas.png")
        .unwrap()
        .into_rgb8();

    let seas_grid = load_gridmap_identifiable_auto(&seas_img, &mut collection, &builder).unwrap();

    let roads_img = image::open("assets/samples/roads.png")
        // let roads_img = image::open("../assets/samples/roads.png")
        .unwrap()
        .into_rgb8();
    let roads_grid = load_gridmap_identifiable_auto(&roads_img, &mut collection, &builder).unwrap();

    // let undertale_img = image::open("assets/samples/undertale.png").unwrap().into_rgb8();
    // let undertale_grid = load_gridmap_identifiable_auto(&undertale_img, &mut collection, &builder).unwrap();

    // Construct WFC Analyzer and provide the maps for analyzing.
    let mut analyzer = WFCAnalyzer::default();
    println!("began analyzing!");
    analyzer.analyze(&seas_grid);
    analyzer.analyze(&roads_grid);
    println!("analysis finished, found: {} tiles", analyzer.tiles().len());
    // analyzer.analyze(&roads_grid);

    // Seed for reproductability.
    // let mut seed: [u8; 32] = [1; 32];
    let mut seed: [u8; 32] = [2; 32];

    for (i, byte) in "wfc_example".as_bytes().iter().enumerate() {
        if i < 31 {
            seed[i] = *byte;
        }
    }

    let mut rng = rand_chacha::ChaChaRng::from_seed(seed);

    // Create new grid via WFC Resolver.
    let size = GridSize::new(50, 50);
    let mut resolver = WFCResolver::new(size, &analyzer);
    println!("populating map!");
    resolver.populate_map_all(&mut rng);

    let mut can_process = true;

    println!("began resolving!");
    while can_process {
        can_process = resolver.process(&mut rng);
        println!("resolved!");
    }

    println!("left with options: {}", resolver.n_with_opts());

    println!(
        "resolved: {} / {}",
        resolver.n_resolved(),
        size.get_all_possible_positions().len()
    );

    let new_map = resolver.build_grid(&builder);

    let mut out_buffer = init_map_image_buffer::<DefaultVisPixel, 4, 4>(&size);
    write_gridmap_identifiable(&mut out_buffer, &new_map, &collection).unwrap();

    out_buffer.save("wfc.png").unwrap();
}
