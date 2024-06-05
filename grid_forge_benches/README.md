Current benchmark results: 

```

running 15 tests
test collapsible::analyze_adjacency_border_10x10   ... bench:      68,616 ns/iter (+/- 5,790)
test collapsible::analyze_adjacency_identity_10x10 ... bench:      21,835 ns/iter (+/- 26,537)
test collapsible::analyze_build_collapsible_grid   ... bench:      19,973 ns/iter (+/- 8,175)
test collapsible::analyze_frequency_10x10          ... bench:       2,238 ns/iter (+/- 400)
test collapsible::gen_border_entrophy_10x10        ... bench:   1,459,970 ns/iter (+/- 245,748)
test collapsible::gen_border_position_10x10        ... bench:   1,127,029 ns/iter (+/- 169,976)
test collapsible::gen_identity_entrophy_10x10      ... bench:   1,450,524 ns/iter (+/- 1,532,590)
test collapsible::gen_identity_position_10x10      ... bench:   1,032,096 ns/iter (+/- 94,169)
test vis_io::load_gridmap_auto                     ... bench:     177,764 ns/iter (+/- 115,718)
test vis_io::load_gridmap_manual                   ... bench:     130,457 ns/iter (+/- 17,995)
test vis_io::write_grimap_ident                    ... bench:      38,594 ns/iter (+/- 24,288)
test walker::walker_grid_4500                      ... bench:      20,569 ns/iter (+/- 824)
test walker::walker_grid_45000                     ... bench:     186,265 ns/iter (+/- 142,626)
test walker::walker_walk_4500                      ... bench:     740,182 ns/iter (+/- 68,765)
test walker::walker_walk_45000                     ... bench:   7,599,618 ns/iter (+/- 797,444)

test result: ok. 0 passed; 0 failed; 0 ignored; 15 measured; 0 filtered out; finished in 50.92s

```
