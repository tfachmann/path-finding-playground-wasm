# Path Finding Playground WASM

A very small playground for testing out path finding algorithms such as Dijkstra.
Implemented with [Yes](https://yew.rs/docs/en/) and [WebAssembly](https://webassembly.org/)

## Building

```sh
wasm-pack build --target web; rollup ./main.js --format iife --file ./pkg/bundle.js
python -m http.server 8080
```

## Usage

Place a Goal first, then put obstacles to change the shortest path
