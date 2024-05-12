extends Node2D

@export var collections: TileCollections;
@export var maps_png_files: Array[String];
@export var generator: TileGenerator;

var load_png_option: OptionButton;
var tilemap: TileMap;

# Called when the node enters the scene tree for the first time.
func _ready():
	tilemap = get_node("TileMapContainer/TileMap");
	load_png_option = get_node("TabContainer/Image/VSplitContainer/OptionButton");
	collections.generate_collections();
	
	var item_id = 0;
	for png_file in maps_png_files:
		var icon = load(png_file);
		load_png_option.add_icon_item(icon, png_file, item_id);
		item_id += 1;
		
	generator.initialize_rulesets(maps_png_files);

func _on_button_pressed():
	var path = load_png_option.get_item_text(load_png_option.selected);
	tilemap.clear();
	collections.convert_png_to_tilemap(path, tilemap);
	adjust_tilemap(true);
	
func adjust_tilemap(use_gap: bool):
	var max_width = 550;
	var max_height = 550;
	var x: int;
	var y: int;
	
	var used = tilemap.get_used_rect();
	if use_gap:
		tilemap.tile_set.tile_size = Vector2i(5, 5);
		x = used.size.x * 5;
		y = used.size.y * 5;
	else:
		tilemap.tile_set.tile_size = Vector2i(4, 4);
		x = used.size.x * 4;
		y = used.size.y * 4;
	
	var x_scale = max_width as float / x as float;
	var y_scale = max_height as float / y as float;
	var final_scale = min(x_scale as int, y_scale as int);
	tilemap.scale = Vector2(final_scale, final_scale);

func _on_tile_map_node_hovered(atlas_coords, tile_pos):
	if atlas_coords == Vector2i(-1, -1):
		get_node("HoverCellPanel/AtlasCoord").clear();
		get_node("HoverCellPanel/TilePos").clear();
	else:
		get_node("HoverCellPanel/AtlasCoord").change_value(atlas_coords);
		get_node("HoverCellPanel/TilePos").change_value(tile_pos);


func _on_button_pressed_gen():
	var width = get_node("TabContainer/Generate/Size/Width/Toggle").value;
	var height = get_node("TabContainer/Generate/Size/Height/Toggle").value;
	var gen_rules = get_node("TabContainer/Generate/Rules").selected;
	var queue = get_node("TabContainer/Generate/Queue").selected;
	
	tilemap.clear();
	if generator.generate(width as int, height as int, gen_rules, queue, tilemap):
		adjust_tilemap(false);
	
