use std::{path::PathBuf, time::Instant};

use inverted_index_coursework::{
    simple_inverted_index::{
        MultiFileThreadedSimpleInvertedIndex, SimpleInvertedIndex, ThreadedSimpleInvertedIndex,
    },
    InvertedIndex,
};
use plotters::prelude::*;

const OUT_FILE_NAME: &'static str = "build_time.png";
const FILES: &[&str] = &["./data/datasets/aclImdb/train/unsup"];
const ITERATIONS: i32 = 10;

fn benchmark_build(num_threads: i32) -> f32 {
    let start = Instant::now();
    let _ = MultiFileThreadedSimpleInvertedIndex::build(
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
    let thread_axis = 1..13;

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
    for thread_num in thread_axis.clone() {
        println!("Threads: {thread_num}");
        let mut times = vec![];

        for _ in iteration_axis.clone() {
            times.push(benchmark_build(thread_num));
        }
        per_thread.push(times);

        let color = Palette99::pick(thread_num as usize);
        cc.draw_series(LineSeries::new(
            iteration_axis
                .clone()
                .map(|x| (x as f32, per_thread[(thread_num - 1) as usize][x as usize])),
            &color,
        ))?
        .label(thread_num.to_string())
        .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));
    }

    cc.configure_series_labels().border_style(&BLACK).draw()?;

    let mut cc = ChartBuilder::on(&lower)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .margin_right(20)
        .caption(format!("Averages"), ("sans-serif", 40))
        .build_cartesian_2d(1f32..12f32, 0f32..5f32)?;
    cc.configure_mesh()
        .x_labels(12)
        .y_labels(20)
        .x_label_formatter(&|v| format!("{:.0}", v))
        .y_label_formatter(&|v| format!("{:.1}", v))
        .draw()?;

    let averages: Vec<f32> = per_thread
        .iter()
        .map(|times| times.iter().sum::<f32>() / times.len() as f32)
        .collect();
    cc.draw_series(LineSeries::new(
        thread_axis
            .clone()
            .map(|x| (x as f32, averages[x as usize - 1])),
        &BLUE,
    ))?;
    // To avoid the IO failure being ignored silently, we manually call the present function
    root_area.present().expect("Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir");
    println!("Result has been saved to {}", OUT_FILE_NAME);
    Ok(())
}
