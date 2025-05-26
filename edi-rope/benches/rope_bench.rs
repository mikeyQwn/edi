use criterion::{Criterion, criterion_group, criterion_main};
use edi_rope::Rope;

const LOREM_SMALL: &str = include_str!("small_ipsum.txt");
const LOREM_MEDIUM: &str = include_str!("medium_ipsum.txt");
const LOREM_BIG: &str = include_str!("big_ipsum.txt");
const POSITIONS: &[(&str, f64)] = &[
    ("start", 0.0),
    ("25%", 0.25),
    ("50%", 0.5),
    ("75%", 0.75),
    ("100%", 1.0),
];

enum RopeSize {
    Empty,
    Small,
    Medium,
    Big,
}

impl RopeSize {
    fn create_fn(&self) -> fn() -> Rope {
        match self {
            RopeSize::Empty => || Rope::new(),
            RopeSize::Small => || Rope::from(LOREM_SMALL),
            RopeSize::Medium => || Rope::from(LOREM_MEDIUM),
            RopeSize::Big => || Rope::from(LOREM_BIG),
        }
    }

    fn content_len(&self) -> usize {
        match self {
            RopeSize::Empty => 0,
            RopeSize::Small => LOREM_SMALL.len(),
            RopeSize::Medium => LOREM_MEDIUM.len(),
            RopeSize::Big => LOREM_BIG.len(),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            RopeSize::Empty => "empty",
            RopeSize::Small => "small",
            RopeSize::Medium => "medium",
            RopeSize::Big => "big",
        }
    }

    fn contents(&self) -> &'static str {
        match self {
            RopeSize::Empty => "",
            RopeSize::Small => LOREM_SMALL,
            RopeSize::Medium => LOREM_MEDIUM,
            RopeSize::Big => LOREM_BIG,
        }
    }
}

const ALL_SIZES: [RopeSize; 4] = [
    RopeSize::Empty,
    RopeSize::Small,
    RopeSize::Medium,
    RopeSize::Big,
];

const NON_EMPTY_SIZES: [RopeSize; 3] = [RopeSize::Small, RopeSize::Medium, RopeSize::Big];

fn bench_rope_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("create");

    for size in ALL_SIZES {
        group.bench_function(size.name(), |b| b.iter(size.create_fn()));
    }
}

fn bench_rope_insert(c: &mut Criterion) {
    let group = &mut c.benchmark_group("insert");

    group.bench_function(format!("at_start_empty"), |b| {
        b.iter_batched(
            RopeSize::Small.create_fn(),
            |mut rope: Rope| {
                rope.insert(0, "hello");
            },
            criterion::BatchSize::SmallInput,
        );
    });

    for (position_name, position_factor) in POSITIONS {
        for size in NON_EMPTY_SIZES {
            let size_name = size.name();
            let insert_index = (position_factor * size.content_len() as f64) as usize;
            group.bench_function(format!("at_{position_name}_{size_name}"), |b| {
                b.iter_batched(
                    size.create_fn(),
                    |mut rope: Rope| {
                        rope.insert(insert_index, "hello");
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }
    }
}

fn bench_lines_skip(c: &mut Criterion) {
    let group = &mut c.benchmark_group("lines_skip");

    group.bench_function(format!("at_start_empty"), |b| {
        b.iter_batched(
            RopeSize::Small.create_fn(),
            |rope: Rope| {
                rope.lines().nth(0);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    for (position_name, position_factor) in POSITIONS {
        for size in NON_EMPTY_SIZES {
            let size_name = size.name();
            let skip_index = (position_factor * size.contents().lines().count() as f64) as usize;
            group.bench_function(format!("at_{position_name}_{size_name}"), |b| {
                b.iter_batched(
                    size.create_fn(),
                    |rope: Rope| {
                        rope.lines().nth(skip_index);
                    },
                    criterion::BatchSize::SmallInput,
                );
            });
        }
    }
}

criterion_group!(
    benches,
    bench_rope_create,
    bench_rope_insert,
    bench_lines_skip
);
criterion_main!(benches);
