#!/usr/bin/env bash

echo -e "Current benchmark results: \n\n\`\`\`" > grid_forge_benches/README.md
cargo bench --package grid_forge_benches --profile release 2>/dev/null 1>> grid_forge_benches/README.md
echo "\`\`\`" >> grid_forge_benches/README.md