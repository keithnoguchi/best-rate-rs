use super::Dex;

#[test]
fn test_direct() {
    let mut dex = Dex::new();
    dex.add_rate('A', 'B', 1.4);
    dex.add_rate('A', 'C', 0.29);
    dex.add_rate('B', 'C', 0.2);

    let src = 'A'.into();
    let dst = 'C'.into();
    let path = dex.get_best_rate(&src, &dst).unwrap();
    assert_eq!(path.rate(), 0.29);
}

#[test]
fn test_one_hop() {
    let mut dex = Dex::new();
    dex.add_rate('A', 'B', 1.4);
    dex.add_rate('A', 'C', 0.1);
    dex.add_rate('B', 'C', 0.2);

    let src = 'A'.into();
    let dst = 'C'.into();
    let path = dex.get_best_rate(&src, &dst).unwrap();
    assert_eq!(path.rate(), 0.28);
}

#[test]
fn test_two_hops() {
    let mut dex = Dex::new();
    dex.add_rate('A', 'B', 1.4);
    dex.add_rate('A', 'C', 0.1);
    dex.add_rate('A', 'D', 0.055);
    dex.add_rate('B', 'C', 0.2);
    dex.add_rate('C', 'D', 0.2);
    dex.add_rate('D', 'F', 2.5);

    let src = 'A'.into();
    let dst = 'D'.into();
    let path = dex.get_best_rate(&src, &dst).unwrap();
    assert_eq!(path.rate(), 0.056);
}

#[test]
fn test_loop() {
    let mut dex = Dex::new();
    dex.add_rate('A', 'B', 1.4);
    dex.add_rate('A', 'C', 0.1);
    dex.add_rate('B', 'C', 0.2);
    dex.add_rate('C', 'D', 0.2);
    dex.add_rate('D', 'F', 2.5);

    let src = 'D'.into();
    let dst = 'F'.into();
    let path = dex.get_best_rate(&src, &dst).unwrap();
    assert_eq!(path.rate(), 2.5);
}
