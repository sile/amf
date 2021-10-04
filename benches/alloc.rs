use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
extern crate amf;

use amf::amf0::Value;

fn write_to_vec_new(value: &Value) {
    let mut buf = Vec::new();
    value.write_to(&mut buf).unwrap();
}

fn write_to_vec_with_capacity(value: &Value) {
    let mut buf = Vec::with_capacity(value.encoded_len());
    value.write_to(&mut buf).unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("allocations");

    let boolean = Value::Boolean(false);
    let string = Value::String("Hello!".to_string());
    let number = Value::Number(10.0);
    let date = Value::Date { unix_time: std::time::Duration::from_millis(0), time_zone:0 };
    let array = Value::Array {
        entries: vec![
            Value::Number(20.0),
            Value::String("Hello!".to_string()),
            Value::Boolean(true),
            Value::Date { unix_time: std::time::Duration::from_millis(0), time_zone:0 }
        ],
    };

    let names = ["boolean", "string", "number", "date", "array"];
    for (x, i) in [boolean, string, number, date, array].iter().enumerate() {
        group.bench_with_input(BenchmarkId::new(names[x], "vec_new"), i, |b, i| {
            b.iter(|| write_to_vec_new(i))
        });
        group.bench_with_input(
            BenchmarkId::new(names[x], "vec_with_capacity"),
            i,
            |b, i| b.iter(|| write_to_vec_with_capacity(i)),
        );
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
