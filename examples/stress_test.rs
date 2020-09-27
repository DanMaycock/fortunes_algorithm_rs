use rand::Rng;

fn main() {
    for _ in 0..1_000 {
        let mut points: Vec<cgmath::Point2<f64>> = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            points.push(cgmath::Point2::new(rng.gen(), rng.gen()));
        }

        fortunes_algorithm::build_voronoi(&points);
    }
}
