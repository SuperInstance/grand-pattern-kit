use grand_pattern_kit::CellGraph;

fn main() {
    let mut graph = CellGraph::<8>::new(120.0);
    let bridge = graph.add_room("bridge");
    let engine = graph.add_room("engine_room");
    graph.add_edge(&bridge, &engine);

    for _ in 0..100 {
        graph.tick();
        graph.gossip();
    }

    println!("{}", graph.summary());
}
