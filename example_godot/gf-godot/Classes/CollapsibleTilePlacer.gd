extends Panel

@export var tilemap: MyMap;
@export var collection: TileCollections;

@onready var table = $Table;

var position_to_set = null;

var buttons: ButtonGroup;
var chosen: SingleTile = null;

var to_collapse: Vector2i;

## Tile types which are to be showned on Collapsible Panel.
var tile_types: Array[bool];

## Signal emitted when the panel exited.
signal panel_quitted();

# Called when the node enters the scene tree for the first time.
func _ready():
	buttons = ButtonGroup.new();
	buttons.connect("pressed", _on_tile_button_selected);
	tile_types = [true, true, true, true];
	refresh();

func show_panel():
	visible = true;
	refresh();

func quit_panel():
	visible = false;
	refresh();
	emit_signal("panel_quitted");

func set_position_to_set(pos: Vector2i):
	position_to_set = pos;
	$Label.text = "Position to set: " + str(pos);

func null_position_to_set():
	position_to_set = null;
	$Label.text = "No position chosen!";

func refresh():
	$SetTile.disabled = false;
	remove_tiles();
	load_tiles();

func load_tiles():
	var tiles = collection.get_tiles();
	for idx in tiles.size():
		var single = (tiles[idx] as SingleTile);
		var can_be_added = true;
		for tile_type_idx in tile_types.size():
			if can_be_added and !tile_types[tile_type_idx] and single.has_type(tile_type_idx):
				can_be_added = false;
		if can_be_added:
			var tilebutton = load("res://Classes/TileButton.tscn").instantiate();
			tilebutton.set_tile(single);
			tilebutton.button_group = buttons;
			table.add_child(tilebutton);

func remove_tiles():
	var grid_tiles = table.get_children();
	while grid_tiles.size() > 0:
		var child = grid_tiles.pop_back();
		child.queue_free();

func _on_type_grass_toggled(toggled_on):
	tile_types[SingleTile.GRASS] = toggled_on;
	refresh();

func _on_type_sand_toggled(toggled_on):
	tile_types[SingleTile.SAND] = toggled_on;
	refresh();

func _on_type_water_toggled(toggled_on):
	tile_types[SingleTile.WATER] = toggled_on;
	refresh();

func _on_type_road_toggled(toggled_on):
	tile_types[SingleTile.ROAD] = toggled_on;
	refresh();

func _on_tile_button_selected(button):
	chosen = (button as TileButton).tile;
	$SetTile.disabled = false;

func _on_set_tile_pressed():
	chosen.insert_into(tilemap, position_to_set);
	tilemap.add_to_collapsed(position_to_set, chosen);
	quit_panel();

func _on_back_pressed():
	quit_panel();

func _on_empty_pressed():
	tilemap.set_cell(0, position_to_set, 1, Vector2i(0, 0));
	tilemap.remove_from_collapsed(position_to_set);
