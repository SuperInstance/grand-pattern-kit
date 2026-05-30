# grand-pattern-kit

A cellular graph of rooms that perceive, predict, get surprised, and gossip. Zero dependencies. Pure Rust.

## The 30-Second Version

Imagine every part of your app is a small room. Each room watches what happens, predicts what comes next, notices when it's wrong, and tells its neighbors. The rooms that talk to each other form a graph. Over time, their feelings sync up and anomalies surface on their own. This library makes that graph.

## The Example

```rust
use grand_pattern_kit::CellGraph;

let mut graph = CellGraph::<8>::new(120.0); // 120 BPM
let bridge = graph.add_room("bridge");
let engine = graph.add_room("engine_room");
graph.add_edge(bridge.clone(), engine.clone());

// Each tick: perceive → predict → surprise → vibe shift → gossip
for _ in 0..100 {
    graph.tick();
    graph.gossip();
}

println!("{}", graph.summary());
// CellGraph: 2 rooms, 1 edges, 200 murmurs, 200 signals
// Fleet vibe magnitude: 3.2147
// Fleet surprise: 0.4821
//   Room 'bridge' (room_0): tick=100, surprise=0.4912
//   Room 'engine_room' (room_1): tick=100, surprise=0.4730
```

## The Pieces

| Crate | What it does |
|---|---|
| [room-cell](https://github.com/SuperInstance/room-cell) | The atom — a room that perceives, predicts, updates vibe |
| [vibe-core](https://github.com/SuperInstance/vibe-core) | 16-dimensional affective state vector |
| [jepa-predict](https://github.com/SuperInstance/jepa-predict) | Prediction + surprise (JEPA-style) |
| [tick-engine](https://github.com/SuperInstance/tick-engine) | The clock — BPM-aware tick scheduling |
| [murmur-protocol](https://github.com/SuperInstance/murmur-protocol) | Lightweight gossip messages between rooms |
| [signal-router](https://github.com/SuperInstance/signal-router) | Typed message bus over edges |
| [grand-pattern-abi](https://github.com/SuperInstance/grand-pattern-abi) | C ABI for polyglot use |

This crate reimplements all of them inline with zero dependencies. Swap in the individual crates when you need more control.

## Why This Architecture

Cells work this way. Immune systems work this way. Cities work this way. Decompose into autonomous units, let them communicate through a graph, let intelligence emerge from the structure. Same code runs on an ESP32 or in the cloud — it's just rooms talking to rooms.

Each tick, every room runs the same loop: **perceive** → **predict** → **measure surprise** → **update vibe** → **gossip to neighbors**. Vibe diffuses across edges. Anomalous rooms surface themselves through high surprise scores. No central controller, no single point of failure.

## Install

```toml
[dependencies]
grand-pattern-kit = "0.1"
```

## Test

```
cargo test
```

## Use

```rust
use grand_pattern_kit::CellGraph;

let mut graph = CellGraph::<16>::new(60.0);
graph.add_room("sensor_a");
graph.tick();
graph.gossip();
```

## License

MIT
