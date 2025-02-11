use meadow::prelude::*;
use rand::Rng;

#[test]
fn host_only_ops() {
    let sc = SledConfig::new().temporary(true);
    let mut host = HostConfig::default().with_sled_config(sc).build().unwrap();
    let mut rng = rand::thread_rng();
    for _i in 0..10 {
        let data: usize = rng.gen_range(0..10);
        host.insert("test", data).unwrap();
        let back: usize = host.get("test").unwrap().data;
        dbg!(&back);
        assert_eq!(data, back);
    }
}

#[test]
fn host_only_back_n() {
    let sc = SledConfig::new().temporary(true);
    let mut host = HostConfig::default().with_sled_config(sc).build().unwrap();
    let mut rng = rand::thread_rng();
    let n = 10;
    for i in 0..n {
        // let data: usize = rng.gen();

        host.insert("test", i).unwrap();
        assert_eq!(host.get::<usize>("test").unwrap().data, i);
        assert_eq!(host.get_nth_back::<usize>("test", 0).unwrap().data, i);
    }
    for _i in 0..n {
        let back = rng.gen_range(1..n);
        let data = n - back;
        // We use "back + 1" because we're zero-indexed
        assert_eq!(
            host.get_nth_back::<usize>("test", back - 1).unwrap().data,
            data
        );
        let result = host.get_nth_back::<usize>("test", 10);
        match result {
            Err(Error::NoNthValue) => (),
            _ => panic!("Should not be okay!"),
        }
    }
}
