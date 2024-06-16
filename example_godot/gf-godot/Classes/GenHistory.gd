extends Panel;

class_name GenHistory;

@export var collection: TileCollections;
var tilemap: TileMap;
var control;
var state: GenerationHistoryState;

# Called when the node enters the scene tree for the first time.
func _ready():
	state = get_node("GenerationHistoryState") as GenerationHistoryState;
	tilemap = get_node("TileMapContainer/TileMap");
	control = get_node("Control");
	state.collection = collection;
	state.tilemap = tilemap;
	
func on_generation_finished(generator: TileGenerator):
	state.set_history_from_generator(generator);
	
func refresh():
	state.current = 0;
	control.set_state(0, state.total);
	state.tilemap.clear();
	state.stop();

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
