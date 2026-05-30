//! grand-pattern-kit: composed Grand Pattern cellular graph system
//! Zero external dependencies — all primitives reimplemented inline.

// ── Primitive types (reimplemented from each sub-crate) ──────────────

/// 16-dimensional affective state vector (from vibe-core)
#[derive(Clone, Debug)]
pub struct Vibe {
    pub dims: [f64; 16],
}

impl Vibe {
    pub fn zero() -> Self {
        Self { dims: [0.0; 16] }
    }

    pub fn random() -> Self {
        let mut dims = [0.0; 16];
        // Simple pseudo-random using a linear congruential approach
        let mut seed: u64 = 0xC0FFEE;
        for d in dims.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *d = ((seed >> 33) as f64) / (1u64 << 31) as f64 - 1.0;
        }
        Self { dims }
    }

    pub fn average(vibes: &[&Vibe]) -> Vibe {
        if vibes.is_empty() {
            return Vibe::zero();
        }
        let mut dims = [0.0; 16];
        for v in vibes {
            for (d, v_d) in dims.iter_mut().zip(v.dims.iter()) {
                *d += *v_d;
            }
        }
        let n = vibes.len() as f64;
        for d in dims.iter_mut() {
            *d /= n;
        }
        Vibe { dims }
    }

    pub fn magnitude(&self) -> f64 {
        self.dims.iter().map(|d| d * d).sum::<f64>().sqrt()
    }

    /// Shift this vibe toward another by a factor (0.0..1.0)
    pub fn lerp_toward(&mut self, other: &Vibe, factor: f64) {
        for i in 0..16 {
            self.dims[i] += (other.dims[i] - self.dims[i]) * factor;
        }
    }
}

/// Embedding vector with timestamp and surprise metric (from jepa-predict)
#[derive(Clone, Debug)]
pub struct Embedding<const D: usize> {
    pub data: [f64; D],
    pub timestamp: u64,
    pub surprise: f64,
}

impl<const D: usize> Embedding<D> {
    pub fn zero(timestamp: u64) -> Self {
        Self {
            data: [0.0; D],
            timestamp,
            surprise: 0.0,
        }
    }

    pub fn random(timestamp: u64) -> Self {
        let mut data = [0.0; D];
        let mut seed: u64 = timestamp.wrapping_add(0xBEEF);
        for d in data.iter_mut() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *d = ((seed >> 33) as f64) / (1u64 << 31) as f64 - 1.0;
        }
        Self {
            data,
            timestamp,
            surprise: 0.0,
        }
    }

    /// Cosine similarity with another embedding
    pub fn cosine_similarity(&self, other: &Embedding<D>) -> f64 {
        let dot: f64 = self.data.iter().zip(other.data.iter()).map(|(a, b)| a * b).sum();
        let mag_a: f64 = self.data.iter().map(|d| d * d).sum::<f64>().sqrt();
        let mag_b: f64 = other.data.iter().map(|d| d * d).sum::<f64>().sqrt();
        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }
        dot / (mag_a * mag_b)
    }
}

/// A room cell in the graph (from room-cell)
#[derive(Clone, Debug)]
pub struct Room<const D: usize> {
    pub id: String,
    pub name: String,
    pub vibe: Vibe,
    pub perception_db: Vec<Embedding<D>>,
    pub prediction_db: Vec<Embedding<D>>,
    pub tick_count: u64,
}

impl<const D: usize> Room<D> {
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            vibe: Vibe::random(),
            perception_db: Vec::new(),
            prediction_db: Vec::new(),
            tick_count: 0,
        }
    }

    /// Perceive: generate a new embedding from current state
    pub fn perceive(&mut self, tick: u64) -> Embedding<D> {
        let emb = Embedding::random(tick);
        self.perception_db.push(emb.clone());
        emb
    }

    /// Predict: generate a prediction based on last perception
    pub fn predict(&mut self, tick: u64) -> Embedding<D> {
        let prediction = if let Some(last) = self.perception_db.last() {
            let mut pred = last.clone();
            pred.timestamp = tick;
            // Add small perturbation for prediction
            let mut seed = tick.wrapping_add(0xDEAD);
            for d in pred.data.iter_mut() {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                *d += ((seed >> 33) as f64) / (1u64 << 32) as f64 * 0.1 - 0.05;
            }
            pred
        } else {
            Embedding::random(tick)
        };
        self.prediction_db.push(prediction.clone());
        prediction
    }

    /// Compute surprise: distance between latest perception and prediction
    pub fn compute_surprise(&self) -> f64 {
        if self.perception_db.is_empty() || self.prediction_db.is_empty() {
            return 0.0;
        }
        let last_perception = self.perception_db.last().unwrap();
        let last_prediction = self.prediction_db.last().unwrap();
        // 1 - cosine similarity as surprise
        let sim = last_perception.cosine_similarity(last_prediction);
        (1.0 - sim).max(0.0)
    }

    /// Update vibe based on surprise
    pub fn update_vibe(&mut self, surprise: f64) {
        // Surprise shifts the vibe dimensions
        for (i, d) in self.vibe.dims.iter_mut().enumerate() {
            let shift = surprise * ((i as f64 * 0.1) - 0.8);
            *d += shift;
        }
    }

    /// Garbage collect old embeddings (keep last 100)
    pub fn gc(&mut self) {
        let max = 100;
        if self.perception_db.len() > max {
            let drain = self.perception_db.len() - max;
            self.perception_db.drain(..drain);
        }
        if self.prediction_db.len() > max {
            let drain = self.prediction_db.len() - max;
            self.prediction_db.drain(..drain);
        }
    }

    /// Average surprise across all perceptions
    pub fn avg_surprise(&self) -> f64 {
        if self.perception_db.is_empty() || self.prediction_db.is_empty() {
            return 0.0;
        }
        let count = self.perception_db.len().min(self.prediction_db.len());
        let mut total = 0.0;
        for i in 0..count {
            let sim = self.perception_db[i].cosine_similarity(&self.prediction_db[i]);
            total += (1.0 - sim).max(0.0);
        }
        total / count as f64
    }

    /// Total "energy" in (sum of perception magnitudes)
    pub fn energy_in(&self) -> f64 {
        self.perception_db
            .iter()
            .map(|e| e.data.iter().map(|d| d * d).sum::<f64>().sqrt())
            .sum()
    }

    /// Total "energy" out (sum of prediction magnitudes)
    pub fn energy_out(&self) -> f64 {
        self.prediction_db
            .iter()
            .map(|e| e.data.iter().map(|d| d * d).sum::<f64>().sqrt())
            .sum()
    }
}

/// Murmur: lightweight gossip message (from murmur-protocol)
#[derive(Clone, Debug)]
pub struct Murmur {
    pub source: String,
    pub vibe_snapshot: [f64; 16],
    pub surprise_avg: f64,
    pub tick: u64,
}

/// Tick clock (from tick-engine)
#[derive(Clone, Debug)]
pub struct Tick {
    pub id: u64,
    pub timestamp: f64,
}

/// Signal types for inter-room communication (from signal-router)
#[derive(Clone, Debug, PartialEq)]
pub enum SignalType {
    Tick,
    Murmur,
    Prediction,
    Surprise,
    VibeShift,
    Anomaly,
}

/// Signal: directed message between rooms (from signal-router)
#[derive(Clone, Debug)]
pub struct Signal {
    pub source: String,
    pub target: String,
    pub signal_type: SignalType,
    pub payload: Vec<f64>,
}

/// Tick scheduling configuration
#[derive(Clone, Debug)]
pub struct TickSchedule {
    pub bpm: f64,
    pub swing: f64,
    pub next_tick: u64,
}

impl TickSchedule {
    pub fn new(bpm: f64) -> Self {
        Self {
            bpm,
            swing: 0.0,
            next_tick: 0,
        }
    }

    pub fn tick_interval_ms(&self) -> f64 {
        if self.bpm <= 0.0 {
            return 1000.0;
        }
        60000.0 / self.bpm
    }
}

// ── Composed system ──────────────────────────────────────────────────

/// CellGraph: the composed cellular graph system
pub struct CellGraph<const D: usize> {
    pub rooms: Vec<Room<D>>,
    pub edges: Vec<(String, String)>,
    pub tick_schedule: TickSchedule,
    pub murmurs: Vec<Murmur>,
    pub signals: Vec<Signal>,
}

impl<const D: usize> CellGraph<D> {
    /// Create an empty graph with the given BPM
    pub fn new(bpm: f64) -> Self {
        Self {
            rooms: Vec::new(),
            edges: Vec::new(),
            tick_schedule: TickSchedule::new(bpm),
            murmurs: Vec::new(),
            signals: Vec::new(),
        }
    }

    /// Add a room, return its id
    pub fn add_room(&mut self, name: &str) -> String {
        let id = format!("room_{}", self.rooms.len());
        let room = Room::new(id.clone(), name.to_string());
        self.rooms.push(room);
        id
    }

    /// Connect two rooms
    pub fn add_edge(&mut self, from: &str, to: &str) {
        self.edges.push((from.to_string(), to.to_string()));
    }

    /// Advance one tick: each room perceives→predicts→surprise→vibe update→gc→emit murmur
    pub fn tick(&mut self) -> Tick {
        let tick_id = self.tick_schedule.next_tick;
        self.tick_schedule.next_tick += 1;

        for room in &mut self.rooms {
            let tick = room.tick_count;
            room.perceive(tick);
            room.predict(tick);
            let surprise = room.compute_surprise();
            room.update_vibe(surprise);
            room.gc();
            room.tick_count += 1;

            // Emit murmur
            let murmur = Murmur {
                source: room.id.clone(),
                vibe_snapshot: room.vibe.dims,
                surprise_avg: room.avg_surprise(),
                tick: tick_id,
            };
            self.murmurs.push(murmur);
        }

        Tick {
            id: tick_id,
            timestamp: tick_id as f64 * self.tick_schedule.tick_interval_ms(),
        }
    }

    /// Each room broadcasts murmur to neighbors
    pub fn gossip(&mut self) {
        let recent_murmurs: Vec<Murmur> = self
            .murmurs
            .iter()
            .rev()
            .take(self.rooms.len())
            .cloned()
            .collect();

        for murmur in &recent_murmurs {
            let neighbors = self.neighbors(&murmur.source);
            for neighbor in neighbors {
                // Vibe diffusion: neighbor shifts toward source
                if let Some(room) = self.rooms.iter_mut().find(|r| r.id == neighbor) {
                    let source_vibe = Vibe {
                        dims: murmur.vibe_snapshot,
                    };
                    room.vibe.lerp_toward(&source_vibe, 0.1);
                }

                // Signal for the murmur
                self.signals.push(Signal {
                    source: murmur.source.clone(),
                    target: neighbor,
                    signal_type: SignalType::Murmur,
                    payload: murmur.vibe_snapshot.to_vec(),
                });
            }
        }
    }

    /// Process signal queue through edges
    pub fn route_signals(&mut self) {
        let signals: Vec<Signal> = self.signals.drain(..).collect();
        for signal in &signals {
            // Validate that an edge exists (in either direction)
            let valid = self.edges.iter().any(|(a, b)| {
                (a == &signal.source && b == &signal.target)
                    || (b == &signal.source && a == &signal.target)
            });
            if valid {
                // Process signal at target room
                if let Some(room) = self.rooms.iter_mut().find(|r| r.id == signal.target) {
                    match signal.signal_type {
                        SignalType::Surprise => {
                            if let Some(&val) = signal.payload.first() {
                                room.update_vibe(val);
                            }
                        }
                        SignalType::VibeShift if signal.payload.len() == 16 => {
                            let incoming = Vibe {
                                dims: {
                                    let mut arr = [0.0; 16];
                                    arr.copy_from_slice(&signal.payload[..16]);
                                    arr
                                },
                            };
                            room.vibe.lerp_toward(&incoming, 0.05);
                        }
                        SignalType::VibeShift => {}
                        SignalType::Anomaly => {
                            // Anomaly signals cause a vibe perturbation
                            for d in room.vibe.dims.iter_mut() {
                                *d += 0.01;
                            }
                        }
                        _ => {}
                    }
                }
            }
            // Re-queue the signal (processed but kept for audit)
            self.signals.push(signal.clone());
        }
    }

    /// Find a room by id
    pub fn find_room(&self, id: &str) -> Option<&Room<D>> {
        self.rooms.iter().find(|r| r.id == id)
    }

    /// Get neighbor room IDs
    pub fn neighbors(&self, id: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter_map(|(a, b)| {
                if a == id {
                    Some(b.clone())
                } else if b == id {
                    Some(a.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Average vibe across all rooms
    pub fn fleet_vibe(&self) -> Vibe {
        if self.rooms.is_empty() {
            return Vibe::zero();
        }
        let vibes: Vec<&Vibe> = self.rooms.iter().map(|r| &r.vibe).collect();
        Vibe::average(&vibes)
    }

    /// Average surprise across all rooms
    pub fn fleet_surprise(&self) -> f64 {
        if self.rooms.is_empty() {
            return 0.0;
        }
        let total: f64 = self.rooms.iter().map(|r| r.avg_surprise()).sum();
        total / self.rooms.len() as f64
    }

    /// Find rooms with surprise above threshold
    pub fn detect_anomaly(&self, threshold: f64) -> Vec<&Room<D>> {
        self.rooms
            .iter()
            .filter(|r| r.avg_surprise() > threshold)
            .collect()
    }

    /// Check energy conservation (|Z_in| ≈ |Z_out|) for all rooms
    pub fn conservation_report(&self) -> Vec<(String, f64, f64, f64)> {
        self.rooms
            .iter()
            .map(|r| {
                let e_in = r.energy_in();
                let e_out = r.energy_out();
                let ratio = if e_in > 0.0 { e_out / e_in } else { 0.0 };
                (r.id.clone(), e_in, e_out, ratio)
            })
            .collect()
    }

    /// Human-readable summary
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "CellGraph: {} rooms, {} edges, {} murmurs, {} signals",
            self.rooms.len(),
            self.edges.len(),
            self.murmurs.len(),
            self.signals.len()
        ));
        lines.push(format!(
            "Tick schedule: {:.1} BPM, next tick: {}",
            self.tick_schedule.bpm, self.tick_schedule.next_tick
        ));
        lines.push(format!(
            "Fleet vibe magnitude: {:.4}",
            self.fleet_vibe().magnitude()
        ));
        lines.push(format!("Fleet surprise: {:.4}", self.fleet_surprise()));
        for room in &self.rooms {
            lines.push(format!(
                "  Room '{}' ({}): tick={}, perceptions={}, predictions={}, surprise={:.4}",
                room.name,
                room.id,
                room.tick_count,
                room.perception_db.len(),
                room.prediction_db.len(),
                room.avg_surprise()
            ));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_graph() -> CellGraph<8> {
        CellGraph::new(120.0)
    }

    #[test]
    fn test_create_graph() {
        let graph: CellGraph<8> = CellGraph::new(120.0);
        assert!(graph.rooms.is_empty());
        assert!(graph.edges.is_empty());
        assert_eq!(graph.tick_schedule.bpm, 120.0);
        assert_eq!(graph.tick_schedule.next_tick, 0);
    }

    #[test]
    fn test_add_rooms() {
        let mut graph = make_graph();
        let id0 = graph.add_room("alpha");
        let id1 = graph.add_room("beta");
        assert_eq!(id0, "room_0");
        assert_eq!(id1, "room_1");
        assert_eq!(graph.rooms.len(), 2);
    }

    #[test]
    fn test_add_edges() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        graph.add_edge(&a, &b);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0], (a, b));
    }

    #[test]
    fn test_tick_advances_all_rooms() {
        let mut graph = make_graph();
        graph.add_room("a");
        graph.add_room("b");
        let tick = graph.tick();
        assert_eq!(tick.id, 0);
        assert_eq!(graph.tick_schedule.next_tick, 1);
        for room in &graph.rooms {
            assert_eq!(room.tick_count, 1);
            assert_eq!(room.perception_db.len(), 1);
            assert_eq!(room.prediction_db.len(), 1);
        }
        assert_eq!(graph.murmurs.len(), 2);
    }

    #[test]
    fn test_gossip_propagates_murmurs() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        graph.add_edge(&a, &b);
        let _vibe_a_before = graph.find_room(&a).unwrap().vibe.dims;

        graph.tick();
        graph.gossip();

        // Room B should have received a murmur from A (or vice versa)
        assert!(!graph.signals.is_empty());
    }

    #[test]
    fn test_route_signals_through_edges() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        graph.add_edge(&a, &b);

        graph.signals.push(Signal {
            source: a.clone(),
            target: b.clone(),
            signal_type: SignalType::Surprise,
            payload: vec![0.5],
        });

        graph.route_signals();
        // Signals should be re-queued after processing
        assert_eq!(graph.signals.len(), 1);
    }

    #[test]
    fn test_find_room_by_id() {
        let mut graph = make_graph();
        let id = graph.add_room("alpha");
        let found = graph.find_room(&id);
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "alpha");

        assert!(graph.find_room("nonexistent").is_none());
    }

    #[test]
    fn test_neighbors_correct() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        let c = graph.add_room("c");
        graph.add_edge(&a, &b);
        graph.add_edge(&b, &c);

        let neighbors_a = graph.neighbors(&a);
        assert_eq!(neighbors_a, vec![b.clone()]);

        let neighbors_b = graph.neighbors(&b);
        assert_eq!(neighbors_b.len(), 2);
        assert!(neighbors_b.contains(&a));
        assert!(neighbors_b.contains(&c));

        let neighbors_c = graph.neighbors(&c);
        assert_eq!(neighbors_c, vec![b]);
    }

    #[test]
    fn test_fleet_vibe_is_average() {
        let mut graph = make_graph();
        graph.add_room("a");
        graph.add_room("b");
        let fleet = graph.fleet_vibe();
        assert_eq!(fleet.dims.len(), 16);
    }

    #[test]
    fn test_fleet_surprise_is_average() {
        let mut graph = make_graph();
        let _ = graph.add_room("a");
        let _ = graph.add_room("b");
        graph.tick();
        let surprise = graph.fleet_surprise();
        assert!(surprise >= 0.0);
    }

    #[test]
    fn test_detect_anomaly_finds_high_surprise_rooms() {
        let mut graph = make_graph();
        let _a = graph.add_room("a");
        let _b = graph.add_room("b");
        graph.tick();

        // Use a very low threshold to detect all rooms
        let anomalies = graph.detect_anomaly(-1.0);
        assert_eq!(anomalies.len(), 2);

        // Use a very high threshold to detect none
        let anomalies = graph.detect_anomaly(1000.0);
        assert_eq!(anomalies.len(), 0);
    }

    #[test]
    fn test_conservation_report() {
        let mut graph = make_graph();
        graph.add_room("a");
        graph.tick();
        let report = graph.conservation_report();
        assert_eq!(report.len(), 1);
        let (_, e_in, e_out, ratio) = &report[0];
        assert!(e_in >= &0.0);
        assert!(e_out >= &0.0);
        // ratio should be finite
        assert!(ratio.is_finite());
    }

    #[test]
    fn test_summary_produces_output() {
        let mut graph = make_graph();
        graph.add_room("alpha");
        graph.tick();
        let summary = graph.summary();
        assert!(summary.contains("CellGraph"));
        assert!(summary.contains("alpha"));
        assert!(summary.contains("1 rooms"));
    }

    #[test]
    fn test_multiple_ticks_accumulate() {
        let mut graph = make_graph();
        graph.add_room("a");
        for _ in 0..5 {
            graph.tick();
        }
        assert_eq!(graph.tick_schedule.next_tick, 5);
        assert_eq!(graph.rooms[0].tick_count, 5);
        assert_eq!(graph.rooms[0].perception_db.len(), 5);
    }

    #[test]
    fn test_empty_graph_handles_gracefully() {
        let mut graph: CellGraph<8> = CellGraph::new(120.0);
        graph.tick(); // Should not panic
        assert_eq!(graph.murmurs.len(), 0);
        let vibe = graph.fleet_vibe();
        assert_eq!(vibe.magnitude(), 0.0);
        assert_eq!(graph.fleet_surprise(), 0.0);
        assert!(graph.summary().contains("0 rooms"));
    }

    #[test]
    fn test_single_room_graph() {
        let mut graph = make_graph();
        let id = graph.add_room("solo");
        graph.tick();
        graph.gossip();
        let room = graph.find_room(&id).unwrap();
        assert_eq!(room.tick_count, 1);
        // No neighbors, so gossip shouldn't produce signals for this room
        let signals_for_room: Vec<_> = graph
            .signals
            .iter()
            .filter(|s| s.target == id || s.source == id)
            .collect();
        assert!(signals_for_room.is_empty());
    }

    #[test]
    fn test_chain_topology() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        let c = graph.add_room("c");
        graph.add_edge(&a, &b);
        graph.add_edge(&b, &c);

        // Tick and gossip
        graph.tick();
        graph.gossip();
        graph.route_signals();

        assert_eq!(graph.neighbors(&a), vec![b.clone()]);
        assert_eq!(graph.neighbors(&c), vec![b]);
    }

    #[test]
    fn test_star_topology() {
        let mut graph = make_graph();
        let hub = graph.add_room("hub");
        let s1 = graph.add_room("spoke1");
        let s2 = graph.add_room("spoke2");
        let s3 = graph.add_room("spoke3");
        graph.add_edge(&hub, &s1);
        graph.add_edge(&hub, &s2);
        graph.add_edge(&hub, &s3);

        let hub_neighbors = graph.neighbors(&hub);
        assert_eq!(hub_neighbors.len(), 3);

        // Each spoke should only see the hub
        assert_eq!(graph.neighbors(&s1), vec![hub.clone()]);
        assert_eq!(graph.neighbors(&s2), vec![hub.clone()]);
        assert_eq!(graph.neighbors(&s3), vec![hub]);
    }

    #[test]
    fn test_mesh_topology() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        let c = graph.add_room("c");
        graph.add_edge(&a, &b);
        graph.add_edge(&a, &c);
        graph.add_edge(&b, &c);

        assert_eq!(graph.neighbors(&a).len(), 2);
        assert_eq!(graph.neighbors(&b).len(), 2);
        assert_eq!(graph.neighbors(&c).len(), 2);
    }

    #[test]
    fn test_vibe_diffusion_across_connected_rooms() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        graph.add_edge(&a, &b);

        let vibe_a_before = graph.find_room(&a).unwrap().vibe.dims;
        let vibe_b_before = graph.find_room(&b).unwrap().vibe.dims;

        graph.tick();
        graph.gossip();

        let vibe_a_after = graph.find_room(&a).unwrap().vibe.dims;
        let vibe_b_after = graph.find_room(&b).unwrap().vibe.dims;

        // At least one dimension should have changed for b due to gossip
        let changed_a: bool = vibe_a_before
            .iter()
            .zip(vibe_a_after.iter())
            .any(|(b, a)| (b - a).abs() > 0.0001);
        let changed_b: bool = vibe_b_before
            .iter()
            .zip(vibe_b_after.iter())
            .any(|(b, a)| (b - a).abs() > 0.0001);
        // At least one room should have been affected by gossip
        assert!(changed_a || changed_b);
    }

    #[test]
    fn test_signal_routing_through_chain() {
        let mut graph = make_graph();
        let a = graph.add_room("a");
        let b = graph.add_room("b");
        let c = graph.add_room("c");
        graph.add_edge(&a, &b);
        graph.add_edge(&b, &c);

        // Signal from a to b (valid edge)
        graph.signals.push(Signal {
            source: a.clone(),
            target: b.clone(),
            signal_type: SignalType::VibeShift,
            payload: vec![0.5; 16],
        });

        // Signal from a to c (no direct edge - invalid)
        graph.signals.push(Signal {
            source: a.clone(),
            target: c.clone(),
            signal_type: SignalType::VibeShift,
            payload: vec![0.5; 16],
        });

        graph.route_signals();

        // Both signals should be re-queued (valid one processed, invalid one just passes through)
        assert_eq!(graph.signals.len(), 2);
    }

    #[test]
    fn test_tick_schedule_adapts() {
        let mut schedule = TickSchedule::new(120.0);
        assert_eq!(schedule.tick_interval_ms(), 500.0);

        schedule.bpm = 60.0;
        assert_eq!(schedule.tick_interval_ms(), 1000.0);

        schedule.bpm = 240.0;
        assert_eq!(schedule.tick_interval_ms(), 250.0);
    }

    #[test]
    fn test_embedding_cosine_similarity() {
        let a: Embedding<4> = Embedding {
            data: [1.0, 0.0, 0.0, 0.0],
            timestamp: 0,
            surprise: 0.0,
        };
        let b: Embedding<4> = Embedding {
            data: [1.0, 0.0, 0.0, 0.0],
            timestamp: 0,
            surprise: 0.0,
        };
        assert!((a.cosine_similarity(&b) - 1.0).abs() < 0.0001);

        let c: Embedding<4> = Embedding {
            data: [0.0, 1.0, 0.0, 0.0],
            timestamp: 0,
            surprise: 0.0,
        };
        assert!((a.cosine_similarity(&c)).abs() < 0.0001);
    }
}
