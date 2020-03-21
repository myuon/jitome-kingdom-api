pub struct RandomGen {}

impl RandomGen {
    // [x,y]
    fn clamp(n: u64, (x, y): (u64, u64)) -> u64 {
        x + n % (y - x)
    }

    pub fn range(x: u64, y: u64) -> u64 {
        RandomGen::clamp(rand::random::<u64>(), (x, y))
    }
}

#[test]
fn test_clamp() {
    let n = RandomGen::clamp(0, (5, 10));
    assert_eq!(n, 5);

    let n = RandomGen::clamp(7, (5, 10));
    assert_eq!(n, 7);

    let n = RandomGen::clamp(5, (5, 10));
    assert_eq!(n, 5);

    let n = RandomGen::clamp(10, (5, 10));
    assert_eq!(n, 10);

    let n = RandomGen::clamp(20, (5, 10));
    assert_eq!(n, 5);
}
