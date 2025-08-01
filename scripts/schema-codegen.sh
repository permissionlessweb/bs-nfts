#!/bin/bash
START_DIR=$(pwd)

echo "🖊 Generating schema...!"
for d in ./contracts/*; do
  if [ -d "$d" ]; then
    for c in "$d"/*; do
      if [ -d "$c" ]; then
        (
          cd "$c" || exit
          cargo run --example schema > /dev/null
          rm -rf ./schema/raw
        )
      fi
    done
  fi
done
echo "✅ Schemas generated."

echo "🖊 Generating TypeScript code...!"
(
  cd ts || exit
  yarn
  yarn run codegen
)
echo "✅ TypeScript code generated."