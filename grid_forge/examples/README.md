# Examples suite

This suite contains only purely Rusty examples of `grid_forge` functionality. Check `example_godot` crate for 
interactive example of `godot` feature.

## `vis.rs`
Basic example of `vis` feature possiblities when creating simple grid images using `grid_forge` and custom `TileData`. 
Most examples use `vis` feature to read and/or write GridMaps.
  
![vis_example](outputs/vis_example.png)

## `gen_walker.rs`
Simple random walk algorithm.

![walker_example](outputs/walker_example.png)

## `collapse`
*Wave-function collapse* or *Model synthesis* generative algorithms, gathered in `grid_forge` under name of *Collapsible
generative algorithms*. Both single-tiled and overlapping
versions. Check [detailed README](collapse/README.md) for more information.

![wfc_example](collapse/outputs/identity_entrophy.gif)