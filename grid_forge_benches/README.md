# Current benchmark results

```

running 18 tests
test collapsible::analyze_adjacency_border_10x10          ... bench:     110,878 ns/iter (+/- 42,707)
test collapsible::analyze_adjacency_identity_10x10        ... bench:      35,913 ns/iter (+/- 3,843)
test collapsible::gen_border_entrophy_10x10               ... bench:  15,094,602 ns/iter (+/- 6,228,863)
test collapsible::gen_border_entrophy_propagate_1_10x10   ... bench:   3,250,876 ns/iter (+/- 525,650)
test collapsible::gen_border_position_10x10               ... bench:     966,397 ns/iter (+/- 209,300)
test collapsible::gen_identity_entrophy_10x10             ... bench:  15,871,107 ns/iter (+/- 5,271,074)
test collapsible::gen_identity_entrophy_propagate_1_10x10 ... bench:   2,597,472 ns/iter (+/- 400,413)
test collapsible::gen_identity_position_10x10             ... bench:     943,086 ns/iter (+/- 713,287)
test overlap::analyze_10x10_pattern_2x2                   ... bench:     132,415 ns/iter (+/- 39,340)
test overlap::analyze_10x10_pattern_3x3                   ... bench:     181,093 ns/iter (+/- 58,369)
test overlap::generate_10x10_pattern_3x3_position         ... bench:   3,498,257 ns/iter (+/- 1,767,547)
test vis_io::load_gridmap_auto                            ... bench:     158,596 ns/iter (+/- 136,686)
test vis_io::load_gridmap_manual                          ... bench:     127,884 ns/iter (+/- 32,481)
test vis_io::write_grimap_ident                           ... bench:      25,554 ns/iter (+/- 11,970)
test walker::walker_grid_4500                             ... bench:      25,097 ns/iter (+/- 9,380)
test walker::walker_grid_45000                            ... bench:     140,534 ns/iter (+/- 69,601)
test walker::walker_walk_4500                             ... bench:     731,311 ns/iter (+/- 159,558)
test walker::walker_walk_45000                            ... bench:   7,458,564 ns/iter (+/- 985,021)

test result: ok. 0 passed; 0 failed; 0 ignored; 18 measured; 0 filtered out; finished in 51.72s

```
