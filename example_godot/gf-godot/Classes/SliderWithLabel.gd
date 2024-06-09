## Simple slider with label, containing the current slider value.

extends HBoxContainer

var label: Label
var slider: HSlider
@export var label_text: String
var slider_value = 0;

func _ready():
	label = get_node("Label");
	slider = get_node("Toggle");
	slider.value_changed.connect(_on_toggle_value_changed);
	
	slider_value = slider.value;
	
	label.text = label_text + ': ' + str(slider_value);

func _process(_delta):
	pass

func _on_toggle_value_changed(value):
	slider_value = value;
	label.text = label_text + ': ' + str(slider_value);
