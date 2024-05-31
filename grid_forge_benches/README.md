Current benchmark results: 

```

running 18 tests
test collapsible::analyze_adjacency_border_10x10          ... bench:      91,697 ns/iter (+/- 12,083)
test collapsible::analyze_adjacency_identity_10x10        ... bench:      17,820 ns/iter (+/- 20,366)
test collapsible::gen_border_entrophy_10x10               ... bench:   4,964,196 ns/iter (+/- 742,839)
test collapsible::gen_border_entrophy_propagate_1_10x10   ... bench:   1,690,770 ns/iter (+/- 307,877)
test collapsible::gen_border_position_10x10               ... bench:     865,245 ns/iter (+/- 81,854)
test collapsible::gen_identity_entrophy_10x10             ... bench:   5,122,452 ns/iter (+/- 787,987)
test collapsible::gen_identity_entrophy_propagate_1_10x10 ... bench:   1,431,820 ns/iter (+/- 161,167)
test collapsible::gen_identity_position_10x10             ... bench:     865,348 ns/iter (+/- 877,158)
test overlap::analyze_10x10_pattern_2x2                   ... bench:     136,147 ns/iter (+/- 27,648)
test overlap::analyze_10x10_pattern_3x3                   ... bench:     193,192 ns/iter (+/- 254,171)
test overlap::generate_10x10_pattern_3x3_position         ... bench:   4,791,455 ns/iter (+/- 1,210,151)
test vis_io::load_gridmap_auto                            ... bench:     151,730 ns/iter (+/- 15,155)
test vis_io::load_gridmap_manual                          ... bench:     128,589 ns/iter (+/- 10,589)
test vis_io::write_grimap_ident                           ... bench:      25,651 ns/iter (+/- 15,729)
test walker::walker_grid_4500                             ... bench:      20,158 ns/iter (+/- 1,222)
test walker::walker_grid_45000                            ... bench:     150,060 ns/iter (+/- 13,621)
test walker::walker_walk_4500                             ... bench:     738,151 ns/iter (+/- 129,473)
test walker::walker_walk_45000                            ... bench:   7,587,998 ns/iter (+/- 1,328,953)

test result: ok. 0 passed; 0 failed; 0 ignored; 18 measured; 0 filtered out; finished in 60.67s

```
