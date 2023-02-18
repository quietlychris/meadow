use meadow::*;

fn main() {
    let node_cfg = NodeConfig::<Tcp, usize>::new("TEAPOT").topic("number");
    let node = node_cfg.build().unwrap();
    dbg!(node);
}
