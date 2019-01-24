use fortunes_algorithm::vector2::Vector2;
use piston_window::*;
use rand::Rng;

const WINDOW_WIDTH: f64 = 720.0;
const WINDOW_HEIGHT: f64 = 720.0;

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
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
const DRAW_VORONOI_VERTICES: bool = true;

const NUM_POINTS: usize = 10_000;

fn diagram_to_canvas(point: &Vector2) -> Vector2 {
    Vector2::new(
        (point.x * (WINDOW_WIDTH - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
        (point.y * (WINDOW_HEIGHT - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
    )
}

fn draw_point<G: Graphics>(point: &Vector2, color: [f32; 4], c: Context, g: &mut G) {
    let point = diagram_to_canvas(point);
    let rectangle = [
        point.x - POINT_SIZE / 2.0,
        point.y - POINT_SIZE / 2.0,
        POINT_SIZE,
        POINT_SIZE,
    ];
    Rectangle::new(color).draw(rectangle, &c.draw_state, c.transform, g);
}

fn draw_edge<G: Graphics>(from: &Vector2, to: &Vector2, color: [f32; 4], c: Context, g: &mut G) {
    let from = diagram_to_canvas(from);
    let to = diagram_to_canvas(to);
    Line::new(color, LINE_WIDTH / 2.0).draw(
        [from.x, from.y, to.x, to.y],
        &c.draw_state,
        c.transform,
        g,
    );
}

fn in_diagram(point: &Vector2) -> bool {
    return point.x >= 0.0 && point.x <= 1.0 && point.y >= 0.0 && point.y <= 1.0;
}

fn draw<G: Graphics>(
    vertices: &[Vector2],
    edges: &[(usize, usize)],
    draw_vertices: bool,
    draw_edges: bool,
    vertex_color: [f32; 4],
    edge_color: [f32; 4],
    c: Context,
    g: &mut G,
) {
    if draw_vertices {
        for vertex in vertices {
            if in_diagram(vertex) {
                draw_point(vertex, vertex_color, c, g);
            }
        }
    }
    if draw_edges {
        for edge in edges {
            draw_edge(&vertices[edge.0], &vertices[edge.1], edge_color, c, g);
        }
    }
}

fn main() {
    let mut points: Vec<Vector2> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..NUM_POINTS {
        points.push(Vector2::new(rng.gen(), rng.gen()));
    }

    points = fortunes_algorithm::lloyds_relaxation(&points, 3);

    let diagram = fortunes_algorithm::generate_diagram(&points);

    let delauney_vertices = fortunes_algorithm::get_delauney_vertices(&diagram);
    let delauney_edges = fortunes_algorithm::get_delauney_edges(&diagram);

    let voronoi_vertices = fortunes_algorithm::get_voronoi_vertices(&diagram);
    let voronoi_edges = fortunes_algorithm::get_voronoi_edges(&diagram);

    let mut window: PistonWindow = WindowSettings::new("Voronoi", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_lazy(true);
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear(BLACK, g);
            draw(
                &delauney_vertices,
                &delauney_edges,
                DRAW_DELAUNEY_VERTICES,
                DRAW_DELAUNEY_EDGES,
                RED,
                YELLOW,
                c,
                g,
            );
            draw(
                &voronoi_vertices,
                &voronoi_edges,
                DRAW_VORONOI_VERTICES,
                DRAW_VORONOI_EDGES,
                BLUE,
                GREEN,
                c,
                g,
            )
        });
    }
}
