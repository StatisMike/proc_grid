## Prints the data for the hovered cell.
extends Label

@export var label: String;

func _ready():
	clear();

## Clear the displayed value: only label will be visible.
func clear():
	text = label;

## Change the value next to the label.
func change_value(value):
	text = label + ": " + str(value);
