## Button allowing to choose the precollapsed cell.
extends Button;
class_name TileButton;

var tile: SingleTile;
@onready var tile_sprite: Sprite2D = $Sprite2D;

func set_tile(init_tile: SingleTile):
	tile = init_tile;

func _ready():
	tile_sprite.texture = ImageTexture.create_from_image(tile.image);
