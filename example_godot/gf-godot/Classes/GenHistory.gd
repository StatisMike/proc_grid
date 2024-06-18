extends Panel;

class_name GenHistory;

@export var collection: TileCollections;

@onready var tilemap: MyMap = $TileMapContainer/TileMap;
@onready var control = $Control;
@onready var state: GenerationHistoryState = $GenerationHistoryState;

## Collection of pregenerated `SingleTile` objects.
var pregenerated = Dictionary();

func _ready():
	## Pass the collection and tilemat to the GenerationHistoryState.
	state.collection = collection;
	state.tilemap = tilemap;

## After generation is finished, the history will be passed all revelant
## information about the run.
func on_generation_finished(generator: TileGenerator):
	state.set_history_from_generator(generator);
	pregenerated = generator.pregenerated;
	insert_pregenerated();

## Insert all precollapsed tiles.
func insert_pregenerated():
	for pos in pregenerated:
		(pregenerated[pos] as SingleTile).insert_into(state.tilemap, pos);

## Refresh the history panel state.
func refresh():
	state.current = 0;
	control.set_state(0, state.total);
	state.tilemap.clear();
	state.stop();
	insert_pregenerated();

func _on_previous_pressed():
	state.backward();

func _on_next_pressed():
	state.forward();

func _on_play_pressed():
	state.play(control.play_start());

func _on_stop_pressed():
	control.play_stop();
	state.stop();
	
func _on_rewind_pressed():
	state.rewind();
	control.set_state(0, state.total);

func _on_generation_history_state_current_state(current):
	control.set_state(current, state.total);
	if current == state.total:
		control.play_stop();

func _on_close_pressed():
	visible = false;
