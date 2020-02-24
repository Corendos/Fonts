use criterion::{black_box, criterion_group, criterion_main, Criterion};
use freetype::Library;

use font::*;

const FONT_SIZE: isize = 24 * 64;
const GLYPHS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789\\|/?.>,<`!@#$%^&*()_-=+[]{};:'\" ";

fn criterion_benchmark(c: &mut Criterion) {
    let library = Library::init().expect("Failed to init freetype library");
    let font_face = library.new_face("/home/corendos/dev/rust/font/resources/fonts/EBGaramond-VariableFont:wght.ttf", 0).expect("Failed to load font");

    c.bench_function("tiling", |b| b.iter(|| {
    
        font_face.set_char_size(0, FONT_SIZE, 0, 72).unwrap();
    
        let _ = FontAtlas::from_str(GLYPHS, &font_face, Some((512, 512))).unwrap();
    }));
}

criterion_group!{
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(30));
    targets = criterion_benchmark
}
criterion_main!(benches);