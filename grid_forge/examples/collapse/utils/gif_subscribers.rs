use std::fs::File;

use grid_forge::{gen::collapse::{overlap::{self, PatternCollection}, singular, CollapsedTileData}, map::GridSize, tile::{identifiable::{builders::ConstructableViaIdentifierTile, BasicIdentTileData}, GridTile}, vis::collection::VisCollection};
use image::{ImageBuffer, Rgb};

pub struct GifSingleSubscriber {
  frame: ImageBuffer<Rgb<u8>, Vec<u8>>,
  encoder: gif::Encoder<File>,
  collection: VisCollection<Rgb<u8>, 4, 4>,
  frame_size: (u16, u16),
}

impl GifSingleSubscriber {
  pub fn new(file: File, size: &GridSize, collection: VisCollection<Rgb<u8>, 4, 4>) -> Self {
    let frame = collection.init_map_image_buffer(size);
    let frame_size = (frame.width() as u16, frame.height() as u16);
    let mut encoder = gif::Encoder::new(file, frame_size.0,frame_size.1, &[]).unwrap();
    
    encoder.set_repeat(gif::Repeat::Infinite).unwrap();

    let mut instance = Self {
      frame,
      encoder,
      collection,
      frame_size,
    };

    instance.write_frame();

    instance
  }

  fn write_frame(&mut self) {
    let buffer = self.frame.clone();
    let frame = gif::Frame::from_rgb_speed(self.frame_size.0, self.frame_size.1, &buffer, 15);
    self.encoder.write_frame(&frame).unwrap();
  }
}

impl singular::Subscriber for GifSingleSubscriber {
  fn on_collapse(&mut self, position: &grid_forge::tile::GridPosition, tile_type_id: u64) {
      self.collection.draw_tile(&GridTile::new(*position, CollapsedTileData::tile_new(tile_type_id)), &mut self.frame).unwrap();
      self.write_frame();
  }
}

impl overlap::Subscriber for GifSingleSubscriber {
  fn on_collapse(&mut self, position: &grid_forge::tile::GridPosition, tile_type_id: u64, pattern_id: u64) {
    self.collection.draw_tile(&GridTile::new(*position, CollapsedTileData::tile_new(tile_type_id)), &mut self.frame).unwrap();
    self.write_frame();
  }
}

// pub struct GifOverlapSubscriber<P: overlap::OverlappingPattern> {
//   frame: ImageBuffer<Rgb<u8>, Vec<u8>>,
//   encoder: gif::Encoder<File>,
//   v_collection: VisCollection<Rgb<u8>, 4, 4>,
//   p_collection: PatternCollection<P>,
//   frame_size: (u16, u16),
// }

// impl GifOverlapSubscriber {
//   pub fn new(file: File, size: &GridSize, v_collection: VisCollection<Rgb<u8>, 4, 4>) -> Self {
//     let frame = collection.init_map_image_buffer(size);
//     let frame_size = (frame.width() as u16, frame.height() as u16);
//     let mut encoder = gif::Encoder::new(file, frame_size.0,frame_size.1, &[]).unwrap();
    
//     encoder.set_repeat(gif::Repeat::Infinite).unwrap();

//     let mut instance = Self {
//       frame,
//       encoder,
//       collection,
//       frame_size,
//     };

//     instance.write_frame();

//     instance
//   }

//   fn write_frame(&mut self) {
//     let buffer = self.frame.clone();
//     let mut frame = gif::Frame::from_rgb_speed(self.frame_size.0, self.frame_size.1, &buffer, 15);
//     self.encoder.write_frame(&frame).unwrap();
//   }
// }

// impl Subscriber for GifSingleSubscriber {
//   fn on_collapse(&mut self, position: &grid_forge::tile::GridPosition, tile_type_id: u64) {
//       self.collection.draw_tile(&GridTile::new(*position, CollapsedTileData::tile_new(tile_type_id)), &mut self.frame).unwrap();
//       self.write_frame();
//   }
// }