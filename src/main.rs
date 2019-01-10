use fortunes_algorithm::vector2::Vector2;
use fortunes_algorithm::voronoi::Voronoi;
use piston_window::*;
use rand::Rng;
use std::time::Instant;

const WINDOW_WIDTH: f64 = 720.0;
const WINDOW_HEIGHT: f64 = 720.0;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.3, 0.3, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

const POINT_SIZE: f64 = 1.0;
const LINE_WIDTH: f64 = 1.0;
const VIEW_MARGIN: f64 = 10.0;

const DRAW_DELAUNEY_EDGES: bool = false;
const DRAW_DELAUNEY_VERTICES: bool = true;
const DRAW_VORONOI_EDGES: bool = true;
const DRAW_VORONOI_VERTICES: bool = false;

const NUM_POINTS: usize = 10_000;

fn diagram_to_canvas(point: Vector2) -> Vector2 {
    Vector2::new(
        (point.x * (WINDOW_WIDTH - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
        (point.y * (WINDOW_HEIGHT - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
    )
}

fn draw_point<G: Graphics>(point: Vector2, color: [f32; 4], c: Context, g: &mut G) {
    let point = diagram_to_canvas(point);
    let rectangle = [
        point.x - POINT_SIZE / 2.0,
        point.y - POINT_SIZE / 2.0,
        POINT_SIZE,
        POINT_SIZE,
    ];
    Rectangle::new(color).draw(rectangle, &c.draw_state, c.transform, g);
}

fn draw_edge<G: Graphics>(from: Vector2, to: Vector2, color: [f32; 4], c: Context, g: &mut G) {
    let from = diagram_to_canvas(from);
    let to = diagram_to_canvas(to);
    Line::new(color, LINE_WIDTH / 2.0).draw(
        [from.x, from.y, to.x, to.y],
        &c.draw_state,
        c.transform,
        g,
    );
}

fn draw_voronoi_diagram<G: Graphics>(diagram: &Voronoi, c: Context, g: &mut G) {
    for face in diagram.get_faces() {
        let start_half_edge = diagram.get_face_outer_component(face).unwrap();
        let mut half_edge = Some(start_half_edge);
        loop {
            if diagram.get_half_edge_origin(half_edge.unwrap()).is_some()
                && diagram
                    .get_half_edge_destination(half_edge.unwrap())
                    .is_some()
            {
                let origin = diagram.get_half_edge_origin(half_edge.unwrap()).unwrap();
                if DRAW_VORONOI_EDGES {
                    let destination = diagram
                        .get_half_edge_destination(half_edge.unwrap())
                        .unwrap();
                    draw_edge(
                        diagram.get_vertex_point(origin),
                        diagram.get_vertex_point(destination),
                        GREEN,
                        c,
                        g,
                    );
                }
                if DRAW_VORONOI_VERTICES {
                    draw_point(diagram.get_vertex_point(origin), RED, c, g);
                }
            }
            half_edge = diagram.get_half_edge_next(half_edge.unwrap());
            if half_edge == Some(start_half_edge) {
                break;
            }
        }
    }
}

fn draw_delauney_diagram<G: Graphics>(diagram: &Voronoi, c: Context, g: &mut G) {
    for (site, _) in diagram.sites.iter() {
        let face = diagram.get_site_face(site).unwrap();
        let start_half_edge = diagram.get_face_outer_component(face).unwrap();
        let mut half_edge = Some(start_half_edge);
        loop {
            if diagram.get_half_edge_twin(half_edge.unwrap()).is_some() {
                if DRAW_DELAUNEY_EDGES {
                    let twin_half_edge = diagram.get_half_edge_twin(half_edge.unwrap()).unwrap();
                    let twin_face = diagram.get_half_edge_incident_face(twin_half_edge);
                    if twin_face.is_some() {
                        let twin_site = diagram.get_face_site(twin_face.unwrap());
                        draw_edge(
                            diagram.get_site_point(site),
                            diagram.get_site_point(twin_site.unwrap()),
                            BLUE,
                            c,
                            g,
                        );
                    }
                }
                if DRAW_DELAUNEY_VERTICES {
                    draw_point(diagram.get_site_point(site), YELLOW, c, g);
                }
            }
            half_edge = diagram.get_half_edge_next(half_edge.unwrap());
            if half_edge == Some(start_half_edge) {
                break;
            }
        }
    }
}

fn main() {
    let mut points: Vec<Vector2> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..NUM_POINTS {
        points.push(Vector2::new(rng.gen(), rng.gen()));
    }

    // points.push(Vector2::new(0.4, 0.2));
    // points.push(Vector2::new(0.6, 0.8));
    // points.push(Vector2::new(0.2, 0.5));
    // points.push(Vector2::new(0.8, 0.5));

    let now = Instant::now();

    let diagram = fortunes_algorithm::generate_diagram(&points);

    let elapsed = now.elapsed();
    println!(
        "Diagram calculated in {} seconds and {} milliseconds",
        elapsed.as_secs(),
        elapsed.subsec_millis()
    );

    let mut window: PistonWindow = WindowSettings::new("Voronoi", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_lazy(true);
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.0, 0.0, 0.0, 1.0], g);
            if DRAW_VORONOI_EDGES || DRAW_VORONOI_VERTICES {
                draw_voronoi_diagram(&diagram, c, g);
            }
            if DRAW_DELAUNEY_EDGES || DRAW_DELAUNEY_VERTICES {
                draw_delauney_diagram(&diagram, c, g);
            }
        });
    }
}
