use std::{path::PathBuf, time::Instant};

use inverted_index_coursework::{simple_inverted_index::SimpleInvertedIndex, InvertedIndex};
use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "build_time.png";
const FILES: &[&str] = &["./data/datasets/aclImdb/train/unsup"];
const ITERATIONS: i32 = 10;

fn benchmark_build(num_threads: i32) -> f32 {
    let start = Instant::now();
    let _ = SimpleInvertedIndex::build(
        FILES.into_iter().map(|s| PathBuf::from(s)).collect(),
        num_threads,
    );

    let duration = Instant::now() - start;

    println!("Iteration done {duration:?}");
    duration.as_secs_f32()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root_area = BitMapBackend::new(OUT_FILE_NAME, (1024, 768)).into_drawing_area();

    root_area.fill(&WHITE)?;
    let root_area = root_area.titled("Build time", ("sans-serif", 60))?;
    let (upper, lower) = root_area.split_vertically(512);

    let iteration_axis = 0..ITERATIONS + 1;

    let mut cc = ChartBuilder::on(&upper)
        .margin(5)
        .set_all_label_area_size(50)
        .caption("Time per iteration", ("sans-serif", 40))
        .build_cartesian_2d(0f32..ITERATIONS as f32, 0f32..5f32)?;

    cc.configure_mesh()
        .x_labels(20)
        .x_desc("Iteration")
        .y_labels(20)
        .y_desc("Time")
        .disable_mesh()
        .x_label_formatter(&|v| format!("{:.0}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    let mut per_thread = vec![];
    for thread_num in 1..2 {
        println!("Threads: {thread_num}");
        let mut times = vec![];

        for _ in iteration_axis.clone() {
            times.push(benchmark_build(thread_num));
        }
        per_thread.push(times);

        cc.draw_series(LineSeries::new(
            iteration_axis
                .clone()
                .map(|x| (x as f32, per_thread[(thread_num - 1) as usize][x as usize])),
            &Palette99::pick(thread_num as usize),
        ))?
        .label(thread_num.to_string());
    }

    cc.configure_series_labels().border_style(&BLACK).draw()?;

    let drawing_areas = lower.split_evenly((1, 2));

    for (drawing_area, idx) in drawing_areas.iter().zip(1..) {
        let mut cc = ChartBuilder::on(&drawing_area)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .margin_right(20)
            .caption(format!("y = x^{}", 1 + 2 * idx), ("sans-serif", 40))
            .build_cartesian_2d(-1f32..1f32, -1f32..1f32)?;
        cc.configure_mesh()
            .x_labels(5)
            .y_labels(3)
            .max_light_lines(4)
            .draw()?;

        cc.draw_series(LineSeries::new(
            (-1f32..1f32)
                .step(0.01)
                .values()
                .map(|x| (x, x.powf(idx as f32 * 2.0 + 1.0))),
            &BLUE,
        ))?;
    }

    // To avoid the IO failure being ignored silently, we manually call the present function
    root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}
