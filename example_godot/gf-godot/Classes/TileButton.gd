extends TextureButton;

class_name TileButton;

var atlas_pos: Vector2i;

func _init(init_image: Image, init_atlas_pos: Vector2i):
	texture_normal = ImageTexture.create_from_image(image);
	

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass
