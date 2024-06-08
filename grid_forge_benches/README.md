Current benchmark results: 

```

running 19 tests
test collapsible::analyze_adjacency_border_10x10   ... bench:      66,968 ns/iter (+/- 28,781)
test collapsible::analyze_adjacency_identity_10x10 ... bench:      16,651 ns/iter (+/- 2,478)
test collapsible::analyze_build_collapsible_grid   ... bench:      19,494 ns/iter (+/- 1,469)
test collapsible::analyze_frequency_10x10          ... bench:       2,261 ns/iter (+/- 189)
test collapsible::gen_border_entrophy_10x10        ... bench:   1,406,750 ns/iter (+/- 91,898)
test collapsible::gen_border_position_10x10        ... bench:   1,085,921 ns/iter (+/- 291,197)
test collapsible::gen_identity_entrophy_10x10      ... bench:   1,364,925 ns/iter (+/- 238,852)
test collapsible::gen_identity_position_10x10      ... bench:     985,996 ns/iter (+/- 80,377)
test overlap::analyze_10x10_pattern_2x2            ... bench:     131,610 ns/iter (+/- 33,376)
test overlap::analyze_10x10_pattern_3x3            ... bench:     162,806 ns/iter (+/- 7,574)
test overlap::generate_10x10_pattern_2x2_entrophy  ... bench:   8,853,598 ns/iter (+/- 627,237)
test overlap::generate_10x10_pattern_3x3_entrophy  ... bench:   7,911,456 ns/iter (+/- 1,420,324)
test vis_io::load_gridmap_auto                     ... bench:     151,972 ns/iter (+/- 124,614)
test vis_io::load_gridmap_manual                   ... bench:     135,141 ns/iter (+/- 37,027)
test vis_io::write_grimap_ident                    ... bench:      29,850 ns/iter (+/- 25,402)
test walker::walker_grid_4500                      ... bench:      21,232 ns/iter (+/- 2,572)
test walker::walker_grid_45000                     ... bench:     148,161 ns/iter (+/- 12,516)
test walker::walker_walk_4500                      ... bench:     713,071 ns/iter (+/- 119,062)
test walker::walker_walk_45000                     ... bench:   7,262,988 ns/iter (+/- 329,490)

test result: ok. 0 passed; 0 failed; 0 ignored; 19 measured; 0 filtered out; finished in 38.09s

```
