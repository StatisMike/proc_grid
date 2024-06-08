extends Label

@export var label: String;

func _ready():
	clear();

func clear():
	text = label;

func change_value(value):
	text = label + ": " + str(value);
