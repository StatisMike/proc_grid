extends TileMap


# Called when the node enters the scene tree for the first time.
func _ready():
	print(get_used_rect())
	for source_idx in tile_set.get_source_count():
		print("source_id: ", source_idx);
		var source = tile_set.get_source(tile_set.get_source_id(source_idx));
		for tile_idx in source.get_scene_tiles_count():
			print(source.get_scene_tile_id(tile_idx));
	var type_layer = tile_set.get_custom_data_layer_by_name('type_id');
	#tile_set.custom_data_layers.
	print(type_layer);
	var coords = get_used_cells(0);
	for coord in coords: 
		print("tile coords:", coord);
		print("atlas coords:", get_cell_atlas_coords(0, coord));
		print("alternative id:", get_cell_alternative_tile(0, coord));



# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
