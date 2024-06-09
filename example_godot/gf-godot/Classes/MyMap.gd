## Slightly modified version of the `TileMap` class, which emits the signal when the mouse is hovered over the tile.
extends TileMap

var last_tile_pos: Vector2i;

signal node_hovered(atlas_coords: Vector2i, tile_pos: Vector2i);

func _process(_delta):
	var mouse_pos_global = get_viewport().get_mouse_position();
	var mouse_pos_local = to_local(mouse_pos_global);
	var tile_pos = local_to_map(mouse_pos_local);
	if tile_pos != last_tile_pos:
		last_tile_pos = tile_pos;
		var atlas_coords = get_cell_atlas_coords(0, tile_pos, 0);
		emit_signal("node_hovered", atlas_coords, tile_pos);
		
	
	
	
	
