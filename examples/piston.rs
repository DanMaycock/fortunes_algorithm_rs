use fortunes_algorithm::delauney::get_delauney_graph;
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

const POINT_SIZE: f64 = 2.0;
const LINE_WIDTH: f64 = 0.5;
const VIEW_MARGIN: f64 = 10.0;

const DRAW_DELAUNEY_EDGES: bool = true;
const DRAW_DELAUNEY_VERTICES: bool = true;
const DRAW_VORONOI_EDGES: bool = false;
const DRAW_VORONOI_VERTICES: bool = false;

const NUM_POINTS: usize = 10_000;

fn diagram_to_canvas(point: &Vector2) -> Vector2 {
    Vector2::new(
        (point.x * (WINDOW_WIDTH - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
        (point.y * (WINDOW_HEIGHT - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
    )
}

fn draw_point<G: Graphics>(point: &Vector2, pen: Rectangle, c: Context, g: &mut G) {
    let point = diagram_to_canvas(point);
    let rectangle = [
        point.x - POINT_SIZE / 2.0,
        point.y - POINT_SIZE / 2.0,
        POINT_SIZE,
        POINT_SIZE,
    ];
    pen.draw(rectangle, &c.draw_state, c.transform, g);
}

fn draw_edge<G: Graphics>(from: &Vector2, to: &Vector2, pen: Line, c: Context, g: &mut G) {
    let from = diagram_to_canvas(from);
    let to = diagram_to_canvas(to);
    pen.draw([from.x, from.y, to.x, to.y], &c.draw_state, c.transform, g);
}

fn in_diagram(point: &Vector2) -> bool {
    point.x >= 0.0 && point.x <= 1.0 && point.y >= 0.0 && point.y <= 1.0
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
    if draw_edges {
        for edge in edges {
            let pen = Line::new(edge_color, LINE_WIDTH / 2.0);
            draw_edge(&vertices[edge.0], &vertices[edge.1], pen, c, g);
        }
    }
    if draw_vertices {
        for vertex in vertices {
            let pen = Rectangle::new(vertex_color);
            if in_diagram(vertex) {
                draw_point(vertex, pen, c, g);
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

    points = fortunes_algorithm::lloyds_relaxation(&points, 5);

    let voronoi = fortunes_algorithm::generate_diagram(&points);

    let delauney = get_delauney_graph(&voronoi);
    let delauney_vertices: Vec<Vector2> = delauney
        .node_indices()
        .map(|node| delauney.node_weight(node).unwrap().position())
        .collect();
    let delauney_edges: Vec<(usize, usize)> = delauney
        .edge_indices()
        .map(|edge| {
            let (from, to) = delauney.edge_endpoints(edge).unwrap();
            (from.index(), to.index())
        })
        .collect();

    let voronoi_vertices = voronoi.get_vertex_points();
    let voronoi_edges = voronoi.get_edge_vertices();

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