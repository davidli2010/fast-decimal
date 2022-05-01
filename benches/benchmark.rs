//! fast decimal benchmark

use bencher::{benchmark_group, benchmark_main, black_box, Bencher};
use fast_decimal::Decimal;

fn decimal_cmp(bench: &mut Bencher) {
    let x: Decimal = "12345678901.23456789".parse().unwrap();
    let y: Decimal = "12345.67890123456789".parse().unwrap();
    bench.iter(|| {
        let _n = black_box(x > y);
    })
}

benchmark_group!(decimal_benches, decimal_cmp,);

benchmark_main!(decimal_benches);
