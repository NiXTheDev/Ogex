use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ogex_core::Regex;

fn bench_basic_matching(c: &mut Criterion) {
    let pattern = Regex::new(r"hello\s+\w+").unwrap();
    let input = "hello world this is a test hello universe";

    c.bench_function("basic_match", |b| {
        b.iter(|| black_box(pattern.find(black_box(input))))
    });
}

fn bench_backreferences(c: &mut Criterion) {
    let pattern = Regex::new(r"(a)(b)\g{-1}").unwrap();
    let input = "abb abc abd abe";

    c.bench_function("backref_match", |b| {
        b.iter(|| black_box(pattern.find(black_box(input))))
    });
}

fn bench_named_groups(c: &mut Criterion) {
    let pattern = Regex::new(r"(name:\w+) is \g{name}").unwrap();
    let input = "John is John and Jane is Jane";

    c.bench_function("named_group_match", |b| {
        b.iter(|| black_box(pattern.find(black_box(input))))
    });
}

fn bench_complex_pattern(c: &mut Criterion) {
    let pattern = Regex::new(r"(email:[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,})").unwrap();
    let input = "Contact us at email:test@example.com or email:admin@company.org";

    c.bench_function("complex_match", |b| {
        b.iter(|| black_box(pattern.find_all(black_box(input))))
    });
}

fn bench_find_all(c: &mut Criterion) {
    let pattern = Regex::new(r"\d+").unwrap();
    let input = "abc 123 def 456 ghi 789 jkl 012 mno 345 pqr 678 stu 901";

    c.bench_function("find_all_numbers", |b| {
        b.iter(|| black_box(pattern.find_all(black_box(input))))
    });
}

fn bench_character_classes(c: &mut Criterion) {
    let pattern = Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap();
    let input = "let x = 42; function_name(); var123 + y";

    c.bench_function("char_class_match", |b| {
        b.iter(|| black_box(pattern.find_all(black_box(input))))
    });
}

criterion_group!(
    benches,
    bench_basic_matching,
    bench_backreferences,
    bench_named_groups,
    bench_complex_pattern,
    bench_find_all,
    bench_character_classes,
);

criterion_main!(benches);
