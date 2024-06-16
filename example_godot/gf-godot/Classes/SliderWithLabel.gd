## Simple slider with label, containing the current slider value.
extends HBoxContainer

class_name SliderWithLabel

@export var label_text: String
@export var default: int
@export var min: int
@export var max: int

var label: Label
var slider: HSlider
var slider_value = 0;

func _ready():
	label = get_node("Label");
	slider = get_node("Toggle") as HSlider;
	
	slider.min_value = min;
	slider.max_value = max;
	slider.step = 1;
	
	_on_toggle_value_changed(default);
	
	slider.value_changed.connect(_on_toggle_value_changed);

func set_value(value):
	_on_toggle_value_changed(value);

func _on_toggle_value_changed(value):
	slider_value = value;
	label.text = label_text + ': ' + str(slider_value);
