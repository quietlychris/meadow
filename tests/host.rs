use meadow::prelude::*;
use rand::{random, Rng};

#[test]
fn host_only_ops() {
    let sc = SledConfig::new().temporary(true);
    let mut host = HostConfig::default().with_sled_config(sc).build().unwrap();
    let mut rng = rand::thread_rng();
    for i in 0..10 {
        let data: usize = rng.gen();
        host.insert("test", data).unwrap();
        let back: usize = host.get("test").unwrap().data;
        dbg!(&back);
        assert_eq!(data, back);
    }
}
