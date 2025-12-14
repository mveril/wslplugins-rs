use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

// Replace `my_crate` with your real crate name (the one in Cargo.toml).
use wslplugins_rs::user_distribution_id::fmt::GuidFormatter;
#[cfg(feature = "uuid")]
use wslplugins_rs::user_distribution_id::fmt::UuidFormatter;
use wslplugins_rs::UserDistributionID;

/// Returns a sample `UserDistributionID` used for all benchmarks.
///
/// Adapt this function to match your real API if needed.
/// For example, if `UserDistributionID` does not implement `FromStr`,
/// construct it from a GUID / `uuid::Uuid` / bytes instead.
#[allow(clippy::expect_used, reason = "the GUID is corect")]
fn sample_user_distribution_id() -> UserDistributionID {
    "00112233-4455-6677-8899-AABBCCDDEEFF"
        .parse()
        .expect("valid UserDistributionID")
}

/// Benchmarks `LowerHex` formatting (`{:x}`) for both formatters.
fn bench_lower_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("user_distribution_id_lower_hex");

    // Setup: build the input only once.
    let id = sample_user_distribution_id();

    // UuidFormatter + {:x}
    #[cfg(feature = "uuid")]
    group.bench_function("uuid_formatter_lower_hex", |b| {
        b.iter(|| {
            let formatter = UuidFormatter::from(black_box(id));
            // Format into a String so work is really performed.
            let s = format!("{formatter:x}");
            black_box(s);
        });
    });

    // GuidFormatter + {:x}
    group.bench_function("guid_formatter_lower_hex", |b| {
        b.iter(|| {
            let formatter = GuidFormatter::from(black_box(id));
            let s = format!("{formatter:x}");
            black_box(s);
        });
    });

    group.finish();
}

/// Benchmarks `UpperHex` formatting (`{:X}`) for both formatters.
fn bench_upper_hex(c: &mut Criterion) {
    let mut group = c.benchmark_group("user_distribution_id_upper_hex");

    let id = sample_user_distribution_id();

    // UuidFormatter + {:X}
    #[cfg(feature = "uuid")]
    group.bench_function("uuid_formatter_upper_hex", |b| {
        b.iter(|| {
            let formatter = UuidFormatter::from(black_box(id));
            let s = format!("{formatter:X}");
            black_box(s);
        });
    });

    // GuidFormatter + {:X}
    group.bench_function("guid_formatter_upper_hex", |b| {
        b.iter(|| {
            let formatter = GuidFormatter::from(black_box(id));
            let s = format!("{formatter:X}");
            black_box(s);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_lower_hex, bench_upper_hex);
criterion_main!(benches);
