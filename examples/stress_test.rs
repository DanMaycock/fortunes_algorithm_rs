use rand::Rng;
use fortunes_algorithm::vector2::Vector2;

fn main() {
    for _ in 0..1_000 {

        let mut points: Vec<Vector2> = vec![];
        let mut rng = rand::thread_rng();
        for _ in 0..10_000 {
            points.push(Vector2::new(rng.gen(), rng.gen()));
        }
        
        fortunes_algorithm::generate_diagram(&points);
    }
}
