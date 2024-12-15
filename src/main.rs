use clap::Parser;
use perf_sched_plot::ParsedData;
use std::io::BufRead;
use std::process::Command;
use textplots::{Chart, Plot, Shape};

#[derive(clap::Parser)]
struct App {
    #[clap(short, long)]
    pid: Option<u64>,
    #[clap(short, long)]
    thread_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let app: App = App::parse();

    //sudo perf sched timehist
    let mut command = Command::new("sudo")
        .arg("perf")
        .arg("sched")
        .arg("timehist")
        .stdout(std::process::Stdio::piped())
        .spawn()?;

    let stdout = command.stdout.take().unwrap();
    let mut reader = std::io::BufReader::new(stdout);

    let mut line = String::with_capacity(1024);

    let mut points = Vec::new();

    while reader.read_line(&mut line)? != 0 {
        let Ok(data) = ParsedData::from_str(&line) else {
            continue;
        };
        if let Some(pid) = app.pid {
            if data.tid != pid {
                continue;
            }
        }

        if let Some(thread_name) = &app.thread_name {
            if data.task_name != *thread_name {
                continue;
            }
        }
        let len = points.len();
        points.push((len as f32, data.sch_delay_ms as f32));

        line.clear();
    }

    let (min, max) = points
        .iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), (_, v)| {
            (min.min(*v), max.max(*v))
        });

    let dist = textplots::utils::histogram(&points, min, max, 10);

    println!("Y=sched delay in ms");
    Chart::new(180, 60, min, max)
        .lineplot(&Shape::Bars(&dist))
        .nice();

    Ok(())
}
