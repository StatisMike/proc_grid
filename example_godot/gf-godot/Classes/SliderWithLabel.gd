extends HBoxContainer

var label: Label
var slider: HSlider
@export var label_text: String
var slider_value = 0;

# Called when the node enters the scene tree for the first time.
func _ready():
	label = get_node("Label");
	slider = get_node("Toggle");
	slider.value_changed.connect(_on_toggle_value_changed);
	
	slider_value = slider.value;
	
	label.text = label_text + ': ' + str(slider_value);

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(_delta):
	pass

func _on_toggle_value_changed(value):
	slider_value = value;
	label.text = label_text + ': ' + str(slider_value);
