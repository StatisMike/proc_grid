extends Node2D

## Collections containing both PNG and Godot representation.
@export var collections: TileCollections;
## List of PNG files to be converted to tilemap.
@export var maps_png_files: Array[String];
## Generator for the tilemap.
@export var generator: TileGenerator;

var load_png_option: OptionButton;
var tilemap: MyMap;
var generation_subscribed = false;
var generation_size: Vector2i;
var tilemap_available_size: Vector2i;
var tab: TabContainer;

const TAB_LOAD = 0;
const TAB_GEN_SINGLE = 1;

# Called when the node enters the scene tree for the first time.
func _ready():
	tilemap = get_node("TileMapContainer/TileMap") as MyMap;
	load_png_option = get_node("TabContainer/Image/VSplitContainer/OptionButton");
	tab = get_node("TabContainer");
	collections.generate_collections();
	
	var item_id = 0;
	for png_file in maps_png_files:
		var icon = load(png_file);
		load_png_option.add_icon_item(icon, png_file, item_id);
		item_id += 1;
		
	generator.initialize_rulesets(maps_png_files);
	
	var tilemap_container = get_node("TileMapContainer");
	tilemap_available_size = Vector2i();
	tilemap_available_size.x = tilemap_container.size.x - tilemap.position.x - 5;
	tilemap_available_size.y = tilemap_container.size.y - tilemap.position.y - 5;
	
	tilemap.ready_for_load();

## Reaction on pressing the *Loading from PNG* button
func _on_button_pressed():
	var path = load_png_option.get_item_text(load_png_option.selected);
	collections.convert_png_to_tilemap(path, tilemap);
	tilemap.adjust();
	
## Adjusting the tilemap to fit available space (*Loading from PNG* version)
func adjust_tilemap():
	var max_width = 550;
	var max_height = 550;
	var x: int;
	var y: int;
	
	var used = tilemap.get_used_rect();
	tilemap.tile_set.tile_size = Vector2i(5, 5);
	x = used.size.x * 5;
	y = used.size.y * 5;
	
	var x_scale = max_width as float / x as float;
	var y_scale = max_height as float / y as float;
	var final_scale = min(x_scale, y_scale);
	var adjusted_scale = snappedf(final_scale, 0.1);
	if (adjusted_scale > final_scale):
		adjusted_scale -= 0.1;
	
	tilemap.scale = Vector2(adjusted_scale, adjusted_scale);
	
## Adjusting the tilemap to fit available space (*Collapsible generation* version)
func adjust_generation_tilemap():
	tilemap.tile_set.tile_size = Vector2i(4, 4);
	var x_scale = tilemap_available_size.x as float / (generation_size.x * 4) as float;
	var y_scale = tilemap_available_size.y as float / (generation_size.y * 4) as float;
	var final_scale = min(x_scale, y_scale);
	var adjusted_scale = snappedf(final_scale, 0.1);
	if (adjusted_scale > final_scale):
		adjusted_scale -= 0.1;

	tilemap.scale = Vector2(adjusted_scale, adjusted_scale);

## Showing the information about individual tile
func _on_tile_map_node_hovered(atlas_coords, tile_pos, source_id):
	if atlas_coords == Vector2i(-1, -1):
		get_node("HoverCellPanel/AtlasCoord").clear();
		get_node("HoverCellPanel/TilePos").clear();
	else:
		if source_id == 0:
			get_node("HoverCellPanel/AtlasCoord").change_value(atlas_coords);
		else:
			get_node("HoverCellPanel/AtlasCoord").change_value("Not collapsed tile. Click tile to collapse!");
		get_node("HoverCellPanel/TilePos").change_value(tile_pos);

## Reaction on pressing the *Generate* button for *Collapsible generation*
func _on_button_pressed_gen():
	var width = get_node("TabContainer/Generate/Size/Width/Toggle").value;
	var height = get_node("TabContainer/Generate/Size/Height/Toggle").value;
	var gen_rules = get_node("TabContainer/Generate/Rules").selected;
	var queue = get_node("TabContainer/Generate/Queue").selected;
	
	tilemap.adjust_generation(Vector2i(width, height));
	generator.begin_generation(width as int, height as int, gen_rules, queue);
	(get_node("TabContainer/Generate/GenerateButton") as Button).disabled = true;

## Final during the generation.
func _on_tile_generator_generation_error(message):
	(get_node("AcceptDialog") as AcceptDialog).dialog_text = message;
	(get_node("AcceptDialog") as AcceptDialog).visible = true;

## Runtime error - the generation will retry.
func _on_tile_generator_generation_runtime_error(message):
	(get_node("TabContainer/Generate/RuntimeError") as Label).text = message;

## After the whole generation is finished.
func _on_tile_generator_generation_finished(success):

	## Tilemap needs to be build only if the generation were not subscribed to.
	if success:
		tilemap.clear();
		generator.generated_to_tilemap(tilemap);
		(get_node("TabContainer/Generate/HistoryButton") as Button).disabled = false;
		(get_node("GenerationHistory") as GenHistory).on_generation_finished(get_node("TileGenerator"));
	
	(get_node("TabContainer/Generate/RuntimeError") as Label).text = "";
	(get_node("TabContainer/Generate/GenerateButton") as Button).disabled = false;
	
## For subscription during the generation - after every collapse tile, the collapsed tile will be inserted into the tilemap.
func _on_tile_generator_generation_collapsed(coords, tile_type_id):
	collections.insert_tile(tilemap, tile_type_id, coords);

func _on_tab_container_tab_changed(changed_tab):
	if changed_tab == TAB_LOAD:
		tilemap.ready_for_load();
	if changed_tab == TAB_GEN_SINGLE:
		tilemap.ready_for_generation(Vector2i(10,10));
		(get_node("TabContainer/Generate/Size/Width") as SliderWithLabel).set_value(10);
		(get_node("TabContainer/Generate/Size/Height") as SliderWithLabel).set_value(10);

func _on_size_button_pressed():
	var width = get_node("TabContainer/Generate/Size/Width/Toggle").value;
	var height = get_node("TabContainer/Generate/Size/Height/Toggle").value;
	tilemap.ready_for_generation(Vector2i(width as int, height as int));


func _on_history_button_pressed():
	var history = get_node("GenerationHistory") as GenHistory;
	history.visible = true;
	history.refresh();
