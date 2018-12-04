extern crate log;
extern crate voronoi;

use voronoi::vector2::Vector2;

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

fn main() {
    init().expect("Failed to initialise logger");

    let points: Vec<Vector2> = vec![
        Vector2::new(0.7, 0.5),
        Vector2::new(0.2, 0.2),
        Vector2::new(0.4, 0.3),
        Vector2::new(0.8, 0.9),
    ];

    let diagram = voronoi::generate_diagram(&points);
}
