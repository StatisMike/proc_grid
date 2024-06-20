# grid-forge

Generic abstraction for grid maps.

I've found it pretty frustrating that every engine of framework have their own way of handling grid maps. This made any attempts at generic
solutions for common problems not easily applicable.

`grid-forge` tries to solve this problem, providing a generic abstraction for grid maps - currently only 2D rectangular grids are supported,
3D grids are planned as well, and possibly support for other shapes in the future.

## Basic functionality

Data hold for each tile in the grid is very flexible, and can be anything that is needed. Just implement a `TileData` marker trait and you are good to go.

```rust
use grid_forge::TileData;

enum TileColor {
    Blue,
    Green,
}

struct TwoColoredTile {
    color: TileColor,
}

impl TileData for TwoColoredTile {}
```

Such data can then be placed inside of the `GridMap2D`, and used hovewer you want. Outside of the grid, the data is always contained
within one of the `TileContainer` implementors:

- `GridTile<T>` - container owning the data for a tile.
- `GridRef<T>` - reference to the data for a tile.
- `GridRefMut<T>` - mutable reference to the data for a tile.

```rust
let size = GridSize::new_xy(100, 100);
let mut map = GridMap2D::<TwoColoredTile>::new(size);
let mut rng = thread_rng();

for pos in map.size().get_all_possible_positions() {
    let color = if rng.gen_bool(0.5) {
        TileColor::Blue
    } else {
        TileColor::Green
    };
    let tile = GridTile::new(pos, TwoColoredTile { color });
    map.insert_tile(tile);
}
```

It can be then retrieved from the GridMap in such a container, holding alongside your data the position of the tile.

Its tends to be useful when you would want to pass the data further through the function - information about its position is often
crucial.

```rust
let pos = GridPosition::new_xy(10, 10);
let tile: GridRef<TwoColoredTile> = map.get_tile_at_position(&pos);

assert_eq!(tile.grid_position(), pos);

let mut tile: GridRefMut<TwoColoredTile> = map.get_mut_tile_at_position(&pos);
assert_eq!(tile.grid_position(), pos);

tile.as_mut().color = TileColor::Blue;
```

### Identifiable tile data

There are often times when you want to use a finite set of tiles, which share some common baseline properties - and often holding all of these properties inside of the grid is not desirable.

For this reason, there is a `IdentifiableTileData` trait - holding an unique identifier for each *tile type*.

```rust
struct TwoColoredTile {
    tile_type_id: u64,
    color: TileColor,
}

impl IdentifiableTileData for TwoColoredTile {
    fn tile_type_id(&self) -> u64 {
        self.tile_type_id
    }
}
```

This allows for storing additional data specific to each tile type in some other container (e.g. one implementing provided `IdentTileCollection` trait) and also easily creating new tile instance of specific type.

```rust
impl ConstructableViaIdentifierTile for TwoColoredTile {
    fn tile_new(tile_type_id: u64) -> Self {
        if tile_type_id == 0 {
            Self { tile_type_id, color: TileColor::Blue }
        } else {
            Self { tile_type_id, color: TileColor::Green }
        }
    }
}
```

`IdentifiableTileData` is used by some built-in tile types used by different more specialized functionalities.

### Visual representation of tiles

There are some basic visualization methods provided within the `grid-forge`, lying under the `#[vis]` feature flag. These are most useful for loading and saving 2D grid maps from/to image files, and use `image` crate under the hood.

`VisTileData` make it easy to generate visual representation of the tile data *dynamically*, eg. by using some other characteristics of the tile.

```rust
struct TwoColoredTile {
    color: TileColor,
}

// Each tile will be represented by 10x10 RGB pixels.
impl<DefaultVisPixel, 10, 10> VisTileData for TwoColoredTile {
    fn vis_pixels(&self) -> [[DefaultVisPixel; 1]; 1] {
      let pixel = match self.color {
          TileColor::Blue => [[DefaultVisPixel::new([0, 0, 255])]],
          TileColor::Green => [[DefaultVisPixel::new([0, 255, 0])]],
      }
      vec![vec![pixel; 10]; 10]
    }
}
```

There are also options associated with `VisCollection` struct, which holds visual representation of each `IdentifiableTileData` tile type - making it kind of naive and basic resource system.

### Procedural generation

There is a whole procedural generation module, at the moment containing two kind of generators:

- basic *Random Walk algorithm* - see `gen_walker` example.
- collapsible tile generation (Model Synthesis/Wave function collapse) - see `gen_collapse_overlap` and `gen_collapse_singular` examples. 

### Godot integration

`godot` module contains a collection of structs allowing for easy roundtrips between Godot's and `grid-forge` data structures, using `IdentifiableTileData` trait to synchronize the data between the two sources. 

For Rust-Godot communication it uses GDExtension `godot-rust` crate.

See `example_godot` crate for an example of simple Godot App using the `grid-forge` for loading the map from image file and procedural generation, rendering it in Godot's `TileMap` class. 