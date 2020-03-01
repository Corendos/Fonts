use criterion::{black_box, criterion_group, criterion_main, Criterion};
use font::atlas::{AtlasGenerator, AtlasGeneratorOption, AtlasLoadMode, Padding};

use font::*;

const FONT_SIZE: u32 = 24 * 64;
const GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\\|/?.>,<`!@#$%^&*()_-=+[]{};:'\" ";

fn benchmark_1024_subpixel(c: &mut Criterion) {

    let generator = AtlasGenerator::new(
	&"/home/corendos/dev/rust/font/resources/fonts/EBGaramond-Regular.ttf",
	AtlasGeneratorOption::new(1024, 1024, 72, Padding::new(1, 1, 1, 1)),
	AtlasLoadMode::LCD
    );

    c.bench_function("1024_1024_subpixel", |b| b.iter(|| {
	let font_atlas = generator.generate(FONT_SIZE).unwrap();
    }));
}


fn benchmark_1024_gray(c: &mut Criterion) {

    let generator = AtlasGenerator::new(
	&"/home/corendos/dev/rust/font/resources/fonts/EBGaramond-Regular.ttf",
	AtlasGeneratorOption::new(1024, 1024, 72, Padding::new(1, 1, 1, 1)),
	AtlasLoadMode::Gray
    );

    c.bench_function("1024_1024_gray", |b| b.iter(|| {
	let font_atlas = generator.generate(FONT_SIZE).unwrap();
    }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(30));
    targets = benchmark_1024_subpixel, benchmark_1024_gray
}
criterion_main!(benches);
