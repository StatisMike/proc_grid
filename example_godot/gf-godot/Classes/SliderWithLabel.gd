## Simple slider with label, containing the current slider value.
extends HBoxContainer

class_name SliderWithLabel

@export var label_text: String
@export var default: int
@export var min_value: int
@export var max_value: int

@onready var label: Label = $Label;
@onready var slider: HSlider = $Toggle;
var slider_value = 0;

func _ready():
	
	slider.min_value = min_value;
	slider.max_value = max_value;
	slider.step = 1;
	
	_on_toggle_value_changed(default);
	
	slider.value_changed.connect(_on_toggle_value_changed);

func set_value(value):
	_on_toggle_value_changed(value);

func _on_toggle_value_changed(value):
	slider_value = value;
	label.text = label_text + ': ' + str(slider_value);
