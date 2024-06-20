searchState.loadedDescShard("grid_forge", 0, "Generic abstraction for grid maps.\nAll possible directions from tile to tile within a …\nStores type in relation to each direction.\nBasic two-dimensional GridMap.\nPosition of the <code>TileData</code> within a <code>GridMap2D</code>.\nReference to the <code>TileData</code> contained within <code>GridMap2D</code>. …\nMutable reference to the <code>TileData</code> contained within …\nTrait gathering the containers for <code>TileData</code> outside of the …\nMarker trait for structs that can be contained withing …\nGet Position distance from border\nGet Position distance from center.\nDestroys the GridMap, returning all tiles.\nDestroys the GridMap, returning all tiles with their …\nFills empty positions using constructor function.\nFilter the <code>pos</code> vector, removing from it all positions …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGenerative algorithms for procedural generation of …\nGet positions of all tiles that are in the border\nGet positions of all tiles that are occupied within the …\nGet tile at specified position mutably.\nGet tile neighbouring the specified position at specified …\nGet tiles neighbouring the specified position.\nGet all tiles with their positions remapped according to …\nGet tile at specified position.\nThis module provide a way to identify a tile by its …\nInsert tile. Its position will be determined based on …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if each of <code>self</code> dimensions are lesser than or equal …\nTake a step in specified direction from position within …\nCreates new, empty map of given size.\nGet opposite direction.\nAllows operating on image representations of <code>GridMap2D</code>\nCollapsible procedural generation\nError occuring during collapse process.\nTrait shared by objects that handle the selecting …\n<code>GridMap2D</code> containing data of <code>CollapsedTileData</code>.\nSimple <code>TileData</code> containing only the <code>tile_type_id</code>.\nTrait shared by a structs holding a grid of …\nTrait shared by <code>TileData</code> used within collapsible …\nCollapses tiles in a columnwise fashion.\nBasic Subscriber for debugging purposes.\nStarts at the <code>(max, 0)</code> position.\nStarts at the <code>(max, max)</code> position.\nSelect next position to collapse using smallest entrophy …\nA queue that collapses tiles consecutively in a fixed …\nEnum defining the direction in which the tiles will be …\nEnum defining the starting point of the collapse wave.\nCollapses tiles in a rowwise fashion.\nStarts at the <code>(0, 0)</code> position.\nStarts at the <code>(0, max)</code> position.\nCalculate entrophy.\nAssociated function to calculate entrophy.\nAssociated function to calculate entrophy.\nReturns the index of the collapsed option.\nReturns all empty positions in the internal grid.\nReturns all empty positions in the internal grid.\nReturns iteration number when the error occured.\nReturns <code>GridPosition</code> of tile which caused the error.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nPop next position for collapsing.\nChecks if the tile has any possible options.\nChecks if the tile has any possible options.\nInitialize the queue based on provided tiles.\nInserts <code>CollapsedTileData</code> into the specified position in …\nInserts <code>GridTile</code> of <code>CollapsedTileData</code> into the internal …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks if the tile is collapsed.\nChecks if the tile is collapsed.\nReturns <code>true</code> if the error can be solved by retrying the …\nChecks the current size of the inner queue.\nCreates new <code>CollapsedGrid</code> with the given size.\nCreate new collapsed tile data.\nReturns number of possible options for the tile.\nOverlapping collapsible generation\nRemoves all uncollapsed tiles from the internal grid.\nRemoves all uncollapsed tiles from the internal grid.\nRetrieves the collapsed tiles in internal grid as a …\nRetrieves the collapsed tiles in internal grid as a …\nReturns all possitions in the internal grid holds …\nReturns all possitions in the internal grid holds …\nSingular collapsible generation\nReturns iterator over all <code>tile_type_id</code>s of the collapsed …\nUpdate internal based on provided tile.\nAdjacency rules for the <em>overlapping pattern</em>-based …\nGridMap analyzer for overlapping pattern data.\nEvent in the history of tile generation process.\nSimple subscriber to collect history of tile generation …\nTile which can be collapsed into one of mutliple …\nFrequency hints for the <em>overlapping pattern</em>-based …\nTile which contains only information about <code>tile_type_id</code> of …\nPattern used in Overlapping Collapse algorithm.\nOverlappingPattern for two-dimensional grids.\nOverlappingPattern for three-dimensional grids.\nGrid containing pattern data derived from original …\nCollection holding all found patterns found in sample maps.\nTile data of inner grid within <code>OverlappingPatternGrid</code>.\nWhen applied to the struct allows injecting it into …\nTile which besides containing information about …\nSize of the pattern on the <code>x</code> axis.\nSize of the pattern on the <code>y</code> axis.\nSize of the pattern on the <code>z</code> axis.\nAnalyzes the <code>GridMap2D</code> of <code>IdentifiableTileData</code>, gathering …\nAnalyzes the <code>PatternCollection</code> to find out which patterns …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nPrepare new instance out of <code>GridMap2D</code>, populating provided …\nGets <code>tile_type_id</code> for a <code>TileData</code> of a tile present in the …\nReturns history of tile generation process.\nGets a reference to inner <code>GridMap2D</code> containing …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nChecks compatibility between two patterns is specified …\nCalled when a tile is collapsed.\nCalled when the generation process starts.\nCalled when the generation process starts.\nRetrieves pattern identifier.\nRetrieve the subscriber attached to the resolver.\nRetrieves positions of the secondary tiles of the pattern.\nRetrieves <code>tile_type_id</code> of pattern primary tile.\nAdjacency rules for singular collapse algorithm.\nTrait shared by analyzers producing <code>AdjacencyRules</code>.\nAnalyzer creating adjacency rules based on the borders …\nEvent in the history of tile generation process, …\nSimple subscriber to collect history of tile generation …\nTile with options that can be collapsed into one of them. …\nCollapsible grid compatible with <code>singular::Resolver</code>.\nFrequency hints for the <em>adjacency-based</em> generative …\nAnalyzer creating exact adjacency rules on basis of sample …\nResolver of the singular collapsible procedural algorithm.\nWhen applied to the struct allows injecting it into …\nManually add adjacency between two tiles.\nMethod for manual addition of the adjacency between two …\nRetrieves the adjacency rules.\nAnalyzes provided grid map of <code>IdentifiableTileData</code>…\nTo retrieve the concrete subscriber type from …\nChanges the rules for the generation of the tiles and the …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCollapse the <code>CollapsibleTileGrid</code> using <code>EntrophyQueue</code>.\nReturns history of tile generation process.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates a new empty grid with given <code>GridSize</code>, preparing …\nCreates a new grid using the <code>CollapsedGrid</code> as a source …\nCalled when a tile is collapsed.\nCalled when the generation process starts. No-op by …\nCalled when the generation process starts. No-op by …\nPopulates the grid with all collapsed tiles from the …\nRetrieve the subscriber attached to the resolver.\nRetrieves the collection of all <code>tile_type_id</code> which have …\nAttach a subscriber to the resolver. The subscriber will …\nStruct implementing the random walker algorithm, producing …\nNumber of calls to the Self::walk() method.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGenerate GridMap2D out of gathered GridPosition.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nSet up starting position for the walker algorithm.\nSet up maximum step size: at every iteration the Walker …\nSet up minimum step size: at every iteration the Walker …\nProvide the Rng for random generation.\nSet up GridSize for walker to walk inside.\nSpecifies information about given tile in specific <code>TileSet</code>…\nSpecifies information about given tile in specific <code>TileSet</code>…\nError which can occur when working with Godot’s <code>TileMap</code> …\nCollection of <code>GodotTileMapTileInfo</code> identified by their …\nInformation about given tile in specific <code>TileSet</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nIdentifier of <code>alternative_id</code> for this tile data.\nCoordinates of the tile source data within <code>TileSet</code>.\nIdentifier of the <code>TileSetSource</code> within given <code>TileSet</code>.\nIdentifier of the <code>TileSetSource</code> within given <code>TileSet</code>.\nIndex of the tile.\nReturns automatically generated unique identifier for this …\nInserts tile specified by this <code>GodotTileMapTileInfo</code> into …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nLoads <code>GridMap2D</code> from <code>TileMap</code>, automatically loading read …\nLoads <code>GridMap2D</code> from <code>TileMap</code>, using tiles collected in …\nLoad tiles from specific <code>TileSetAtlasSource</code>.\nLoad tile from specific <code>TileSetScenesCollectionSource</code>.\nAutomatically load all tiles contained in all <code>TileSet</code> …\nCreates new <code>GodotTileMapTileInfo</code> for tile in …\nCreates new <code>GodotTileMapTileInfo</code> for tile in …\nWrites <code>GridMap2D</code> to <code>TileMap</code>, using <code>GodotTileMapTileInfo</code> …\nBasic tile struct that implements <code>IdentifiableTileData</code>, …\nIts implementation makes the specific tile identifiable …\n<code>IdentifiableTileData</code> instance builders.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nTrait which allows creating new istance of struct …\nTrait shared by objects which on basis of the grid …\n<code>IdentTileBuilder</code> which creates new tiles of <code>Clone</code>…\n<code>IdentTileBuilder</code> which creates new tiles with given …\n<code>IdentTileBuilder</code> which creates new tiles with given …\nError stemming from missing tiles in <code>IdentTileBuilder</code>.\nProvide tile prototypes to the builder, which will be used …\nCreates tile with given tile identifier at given grid …\nCreates tile with given tile identifier at given grid …\nChecks for missing tile creators out of provided slice of …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nExternal data kept within the collection, bind to the …\nTrait implementing the behaviour for easily keeping and …\nAdds tile data without <code>tile_type_id</code> provided - it will be …\nAdds tile data without <code>tile_type_id</code> provided - it will be …\nAdds <code>data</code> for specified <code>tile_type_id</code>. If given <code>tile_type_id</code>…\nAdds <code>data</code> for specified <code>tile_type_id</code>. If given <code>tile_type_id</code>…\nGenerates <code>tile_type_id</code> using provided <code>DATA</code>.\nGenerates <code>tile_type_id</code> using provided <code>DATA</code>.\nGets <code>DATA</code> stored for given <code>tile_type_id</code>.\nGets <code>DATA</code> stored for given <code>tile_type_id</code>.\nGets <code>tile_type_id</code> bind to given <code>data</code>.\nGets <code>tile_type_id</code> bind to given <code>data</code>.\nExposes inner hashmap. Necessary for other methods …\nExposes inner hashmap mutably. Necessary for other methods …\nAdditional operation that will be done during addition of …\nAdditional operation that will be done during addition of …\nAdditional operation that will be done during removal of …\nAdditional operation that will be done during removal of …\nRemoves tile data if either provided data or hash …\nRemoves tile data if either provided data or hash …\nRemoves <code>DATA</code> for provided <code>tile_type_id</code>. Returns removed …\nRemoves <code>DATA</code> for provided <code>tile_type_id</code>. Returns removed …\nExposes reverse hashmap. Necessary for other methods …\nExposes reverse hashmap mutably. Necessary for other …\nSets tile data without <code>tile_type_id</code> provided - it will be …\nSets tile data without <code>tile_type_id</code> provided - it will be …\nSets <code>data</code> for specified <code>tile_type_id</code>. Returns removed data …\nSets <code>data</code> for specified <code>tile_type_id</code>. Returns removed data …\nDefault pixel type used by <code>grid_forge</code>.\nTrait for retrieving default value for pixels, necessary …\nTrait allowing retrieving pixels directly on basis of …\nVarious IO operations transforming between <code>GridMap2D</code> and …\nReads pixels from image that represents the tile at …\nWrites tile pixels to the image buffer, adjusting the …\nWrites pixels array into image buffer at provided …\nPassed pixels were set successfully.\nPassed pixels were identified as registered for empty tile.\nContains the error value\nPassed pixels were not set, as some were already …\nContains the success value\nPassed pixels were set and overwrote some already …\nCollection of pixels registered for identifiers of tile …\nOutcome of <code>set_*</code> and <code>add_*</code> methods of <code>VisCollection</code>.\nAdd pixels for tiles from <code>GridMap2D</code>, if the tiles …\nAdd pixels for <code>IdentifiableTileData</code>-implementing <code>VisTile2D</code> …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nSets pixels which should be ignored while reading the …\nSet pixels for <code>IdentifiableTileData</code>-implementing <code>VisTile2D</code> …\nError returned by operations on image representations of …\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nChecks the size of the <code>ImageBuffer</code> before writing <code>GridMap2D</code>…\nChecks the size of the <code>ImageBuffer</code> while loading <code>GridMap2D</code> …\nUtility function to generate <code>ImageBuffer</code> of correct size …\nEasily load <code>GridMap2D</code> of <code>IdentifiableTileData</code>-implementing …\nLoad <code>GridMap2D</code> of <code>IdentifiableTileData</code>-implementing struct …\nWrite <code>GridMap2D</code> comprised of tiles containing …\nWrite <code>GridMap2D</code> comprised of tiles containing <code>VisTileData</code> …")