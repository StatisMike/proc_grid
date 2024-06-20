extends Node2D

## Collections containing both PNG and Godot representation.
@export var collections: TileCollections;
## List of PNG-based Texture2Ds to be converted to tilemap.
@export var maps_png_files: Array[String];
## TileMap collapsible generator.
@export var generator: TileGenerator;

@onready var load_png_option: OptionButton = $TabContainer/Image/VSplitContainer/OptionButton;
@onready var tilemap_container = $TileMapContainer;
@onready var tilemap: MyMap = $TileMapContainer/TileMap;
@onready var tab: TabContainer = $TabContainer;

var generation_subscribed = false;
var generation_size: Vector2i;
## Available size for a TileMap
var tilemap_available_size: Vector2i;


## Tab loading TileMap from Image
const TAB_LOAD = 0;
## Tab generating Singular Collapsible
const TAB_GEN_SINGLE = 1;

func _ready():
	collections.generate_collections();
	
	var item_id = 0;
	for png_file in maps_png_files:
		var icon = load(png_file);
		load_png_option.add_icon_item(icon, png_file, item_id);
		item_id += 1;
		
	generator.initialize_rulesets(maps_png_files);
	
	tilemap_available_size = Vector2i();
	tilemap_available_size.x = tilemap_container.size.x - tilemap.position.x - 5;
	tilemap_available_size.y = tilemap_container.size.y - tilemap.position.y - 5;
	
	tilemap.ready_for_load();

## Reaction on pressing the *Loading from PNG* button
func _on_button_pressed():
	var path = load_png_option.get_item_text(load_png_option.selected);
	collections.convert_png_to_tilemap(path, tilemap);
	tilemap.adjust();

## Showing the information about individual tile.
func _on_tile_map_node_hovered(atlas_coords, tile_pos, source_id):
	if atlas_coords == Vector2i(-1, -1):
		$HoverCellPanel/AtlasCoord.clear();
		$HoverCellPanel/TilePos.clear();
	else:
		if source_id == 0:
			$HoverCellPanel/AtlasCoord.change_value(atlas_coords);
		else:
			$HoverCellPanel/AtlasCoord.change_value("Not collapsed tile. Click tile to collapse!");
		
		$HoverCellPanel/TilePos.change_value(tile_pos);

## Reaction on pressing the *Generate* button for *Collapsible generation*
func _on_button_pressed_gen():
	var gen_rules = $TabContainer/Generate/Rules.selected;
	var queue = $TabContainer/Generate/Queue.selected;
	
	tilemap.adjust_generation(generation_size);
	generator.begin_generation(generation_size.x, generation_size.y, gen_rules, queue, tilemap.collapsed);
	$TabContainer/Generate/GenerateButton.disabled = true;

## Last error during the generation.
func _on_tile_generator_generation_error(message):
	$AcceptDialog.dialog_text = message;
	$AcceptDialog.visible = true;

## Runtime error - the generation will retry.
func _on_tile_generator_generation_runtime_error(message):
	$TabContainer/Generate/RuntimeError.text = message;

## After the whole generation is finished.
func _on_tile_generator_generation_finished(success):

	if success:
		tilemap.clear();
		generator.generated_to_tilemap(tilemap);
		$TabContainer/Generate/HistoryButton.disabled = false;
		$GenerationHistory.on_generation_finished(get_node("TileGenerator"));
		tilemap.generated = true;
		$TabContainer/Generate/RuntimeError.text = "Elapsed total: " + str(generator.generation_time_us_total) + " μs\nElapsed successful run: " + str(generator.generation_time_us_success) + " μs";
	else:
		$TabContainer/Generate/RuntimeError.text = "";

	# Make the manual retry possible.
	$TabContainer/Generate/GenerateButton.disabled = false;

## Changing the Tab should change the TileMap accordingly.
func _on_tab_container_tab_changed(changed_tab):
	if changed_tab == TAB_LOAD:
		tilemap.ready_for_load();
	if changed_tab == TAB_GEN_SINGLE:
		tilemap.ready_for_generation(Vector2i(10,10));
		generation_size = Vector2i(10, 10);
		$TabContainer/Generate/Size/Width.set_value(10);
		$TabContainer/Generate/Size/Height.set_value(10);

## Accepting the size will resize the TileMap and propagate collapsible
## tiles, which can be then precollapsed.
func _on_size_button_pressed():
	var width = $TabContainer/Generate/Size/Width/Toggle.value;
	var height = $TabContainer/Generate/Size/Height/Toggle.value;
	generation_size = Vector2i(width as int, height as int);
	tilemap.ready_for_generation(generation_size);

## Execute process history.
func _on_history_button_pressed():
	var history = $GenerationHistory;
	history.visible = true;
	history.refresh();

## Clicking the collapsible tile will show the panel for inserting the tiles.
func _on_tile_map_collapsible_clicked(tile_pos):
	$CollapsibleTilePlacer.set_position_to_set(tile_pos);
	$CollapsibleTilePlacer.show_panel();
	# Make the main tilemap invisible so it won't react on clicks.
	$TileMapContainer/TileMap.visible = false;

## When collapsible tile placer is closed, the main TileMap should be shown.
func _on_collapsible_tile_placer_panel_quitted():
	$TileMapContainer/TileMap.visible = true;
