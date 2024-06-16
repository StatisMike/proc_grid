## Slightly modified version of the `TileMap` class which allows  the signal when the mouse is hovered over the tile.
extends TileMap

class_name MyMap;

var last_tile_pos: Vector2i;
var collapsed: Dictionary = Dictionary();
var collapsible = false;

var available_size = Vector2i(0, 0);

signal node_hovered(atlas_coords: Vector2i, tile_pos: Vector2i, source_id: int);
signal collapsible_clicked(tile_pos: Vector2i);

func _ready():
	var container = get_parent();
	available_size.x = container.size.x - position.x - 5;
	available_size.y = container.size.y - position.y - 5;

func _process(_delta):
	var tile_pos = get_hover_position();
	if tile_pos != last_tile_pos:
		last_tile_pos = tile_pos;
		var atlas_coords = get_cell_atlas_coords(0, tile_pos, 0);
		var source_id = get_cell_source_id(0, tile_pos, 0);
		if source_id > 0:
			emit_signal("node_hovered", atlas_coords, tile_pos, source_id);
		
func get_hover_position():
	var mouse_pos_global = get_viewport().get_mouse_position();
	var mouse_pos_local = to_local(mouse_pos_global);
	return local_to_map(mouse_pos_local);
	
func _input(event):
	if collapsible and event is InputEventMouseButton and event.button_index == MOUSE_BUTTON_LEFT and event.pressed == true:
		var position = get_hover_position();
		if get_cell_source_id(0, position) > 0:
			emit_signal("collapsible_clicked", position);

## Ready the tilemap for loading from image.
func ready_for_load():
	collapsible = false;
	clear();
	
## Ready the tilemap for generation.
func ready_for_generation(size: Vector2i):
	collapsible = true;
	adjust_generation(size);
	for x in size.x:
		for y in size.y:
			set_cell(0, Vector2i(x, y), 1, Vector2i(0, 0));
	
## Adjusting the tilemap to fit available space (*Load image* version)
## Adjustment needs to be made AFTER the loading takes place
func adjust():
	var used = get_used_rect();
	tile_set.tile_size = Vector2i(5, 5);
	var x = used.size.x * 5;
	var y = used.size.y * 5;
	
	var x_scale = available_size.x as float / x as float;
	var y_scale = available_size.y as float / y as float;
	var final_scale = min(x_scale, y_scale);
	var adjusted_scale = snappedf(final_scale, 0.1);
	if (adjusted_scale > final_scale):
		adjusted_scale -= 0.1;
	
	scale = Vector2(adjusted_scale, adjusted_scale);
	
## Adjusting the tilemap to fit available space (*Collapsible generation* version)
## Adjustment made before generation.
func adjust_generation(generation_size: Vector2i):
	tile_set.tile_size = Vector2i(4, 4);
	var x_scale = available_size.x as float / (generation_size.x * 4) as float;
	var y_scale = available_size.y as float / (generation_size.y * 4) as float;
	var final_scale = min(x_scale, y_scale);
	var adjusted_scale = snappedf(final_scale, 0.1);
	if (adjusted_scale > final_scale):
		adjusted_scale -= 0.1;

	scale = Vector2(adjusted_scale, adjusted_scale);
