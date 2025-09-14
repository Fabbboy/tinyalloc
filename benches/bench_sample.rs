use criterion::{
  Criterion,
  criterion_group,
  criterion_main,
};

fn sample(c: &mut Criterion) {
  c.bench_function("sample_benchmark", |b| {
    b.iter(|| {
      let sum: u32 = (1..=100).sum();
      sum
    })
  });
}

criterion_group!(benches, sample);
criterion_main!(benches);
