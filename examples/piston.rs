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

const DRAW_DELAUNEY_EDGES: bool = false;
const DRAW_DELAUNEY_VERTICES: bool = true;
const DRAW_VORONOI_EDGES: bool = true;
const DRAW_VORONOI_VERTICES: bool = true;

const NUM_POINTS: usize = 5_000;

fn diagram_to_canvas(point: &cgmath::Point2<f64>) -> cgmath::Point2<f64> {
    cgmath::Point2::new(
        (point.x * (WINDOW_WIDTH - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
        (point.y * (WINDOW_HEIGHT - 2.0 * VIEW_MARGIN)) + VIEW_MARGIN,
    )
}

fn draw_point<G: Graphics>(point: &cgmath::Point2<f64>, pen: Rectangle, c: Context, g: &mut G) {
    let point = diagram_to_canvas(point);
    let rectangle = [
        (point.x - POINT_SIZE / 2.0) as f64,
        (point.y - POINT_SIZE / 2.0) as f64,
        POINT_SIZE as f64,
        POINT_SIZE as f64,
    ];
    pen.draw(rectangle, &c.draw_state, c.transform, g);
}

fn draw_edge<G: Graphics>(
    from: &cgmath::Point2<f64>,
    to: &cgmath::Point2<f64>,
    pen: Line,
    c: Context,
    g: &mut G,
) {
    let from = diagram_to_canvas(from);
    let to = diagram_to_canvas(to);
    pen.draw(
        [from.x as f64, from.y as f64, to.x as f64, to.y as f64],
        &c.draw_state,
        c.transform,
        g,
    );
}

fn in_diagram(point: &cgmath::Point2<f64>) -> bool {
    point.x >= 0.0 && point.x <= 1.0 && point.y >= 0.0 && point.y <= 1.0
}

fn draw<G: Graphics>(
    vertices: &[cgmath::Point2<f64>],
    edges: &[(cgmath::Point2<f64>, cgmath::Point2<f64>)],
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
            draw_edge(&edge.0, &edge.1, pen, c, g);
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
    let mut points: Vec<cgmath::Point2<f64>> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..NUM_POINTS {
        let candidate = cgmath::Point2::new(rng.gen(), rng.gen());
        points.push(candidate);
    }

    points = fortunes_algorithm::lloyds_relaxation(&points, 5);

    let voronoi = fortunes_algorithm::generate_diagram(&points);

    let delauney = fortunes_algorithm::delauney::get_delauney_graph(&voronoi);
    let delauney_vertices: Vec<cgmath::Point2<f64>> = delauney
        .node_indices()
        .map(|node| delauney.node_weight(node).unwrap().position.into())
        .collect();
    let delauney_edges: Vec<(cgmath::Point2<f64>, cgmath::Point2<f64>)> = delauney
        .edge_indices()
        .map(|edge| {
            let (from, to) = delauney.edge_endpoints(edge).unwrap();
            (
                delauney.node_weight(from).unwrap().position.into(),
                delauney.node_weight(to).unwrap().position.into(),
            )
        })
        .collect();

    let voronoi_vertices = voronoi.get_vertex_points();
    let voronoi_edges = voronoi.get_edge_endpoints();

    let mut window: PistonWindow =
        WindowSettings::new("Voronoi", [WINDOW_WIDTH as f64, WINDOW_HEIGHT as f64])
            .exit_on_esc(true)
            .build()
            .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_lazy(true);
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g, _device| {
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
