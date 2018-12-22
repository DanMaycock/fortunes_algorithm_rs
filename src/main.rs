use piston_window::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use voronoi::vector2::Vector2;
use voronoi::voronoi::Voronoi;

const WINDOW_WIDTH: f64 = 1080.0;
const WINDOW_HEIGHT: f64 = 1080.0;

const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const BLUE: [f32; 4] = [0.3, 0.3, 1.0, 1.0];
const YELLOW: [f32; 4] = [1.0, 1.0, 0.0, 1.0];

const POINT_SIZE: f64 = 1.0;
const LINE_WIDTH: f64 = 1.0;

fn diagram_to_canvas(point: Vector2) -> Vector2 {
    Vector2::new(point.x * WINDOW_WIDTH, point.y * WINDOW_HEIGHT)
}

fn draw_point<G: Graphics>(point: Vector2, color: [f32; 4], c: Context, g: &mut G) {
    let point = diagram_to_canvas(point);
    let location = [point.x - 1.0, point.y - 1.0, 3.0, 3.0];
    Rectangle::new(color).draw(location, &c.draw_state, c.transform, g);
}

fn draw_edge<G: Graphics>(from: Vector2, to: Vector2, color: [f32; 4], c: Context, g: &mut G) {
    let from = diagram_to_canvas(from);
    let to = diagram_to_canvas(to);
    Line::new(color, 0.5).draw([from.x, from.y, to.x, to.y], &c.draw_state, c.transform, g);
}

fn draw_voronoi_diagram<G: Graphics>(diagram: &Voronoi, c: Context, g: &mut G) {
    for (site, _) in diagram.sites.iter() {
        let face = diagram.get_site_face(site).unwrap();
        let mut start_half_edge = diagram.get_face_outer_component(face);
        if start_half_edge.is_some() {
            while diagram
                .get_half_edge_prev(start_half_edge.unwrap())
                .is_some()
            {
                start_half_edge = diagram.get_half_edge_prev(start_half_edge.unwrap());
                if start_half_edge == diagram.get_face_outer_component(face) {
                    break;
                }
            }
        }
        let mut half_edge = start_half_edge;
        while half_edge.is_some() {
            if diagram.get_half_edge_origin(half_edge.unwrap()).is_some()
                && diagram
                    .get_half_edge_destination(half_edge.unwrap())
                    .is_some()
            {
                let origin = diagram.get_half_edge_origin(half_edge.unwrap()).unwrap();
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
            half_edge = diagram.get_half_edge_next(half_edge.unwrap());
            if half_edge == start_half_edge {
                break;
            }
        }
    }
    for vertex in diagram.get_voronoi_vertices() {
        draw_point(vertex, RED, c, g);
    }
}

fn draw_delauney_diagram<G: Graphics>(diagram: &Voronoi, c: Context, g: &mut G) {
    for (site, _) in diagram.sites.iter() {
        let face = diagram.get_site_face(site).unwrap();
        let mut start_half_edge = diagram.get_face_outer_component(face);
        if start_half_edge.is_some() {
            while diagram
                .get_half_edge_prev(start_half_edge.unwrap())
                .is_some()
            {
                start_half_edge = diagram.get_half_edge_prev(start_half_edge.unwrap());
                if start_half_edge == diagram.get_face_outer_component(face) {
                    break;
                }
            }
        }
        let mut half_edge = start_half_edge;
        while half_edge.is_some() {
            let twin_half_edge = diagram.get_half_edge_twin(half_edge.unwrap()).unwrap();
            let twin_face = diagram.get_half_edge_incident_face(twin_half_edge);
            if twin_face.is_some() {
                let twin_site = diagram.get_face_site(twin_face.unwrap());
                draw_edge(
                    diagram.get_site_point(site),
                    diagram.get_site_point(twin_site.unwrap()),
                    YELLOW,
                    c,
                    g,
                );
            }
            half_edge = diagram.get_half_edge_next(half_edge.unwrap());
            if half_edge == start_half_edge {
                break;
            }
        }
    }
}

fn draw_delauney_vertices<G: Graphics>(diagram: &Voronoi, c: Context, g: &mut G) {
    for vertex in diagram.get_delauney_vertices() {
        draw_point(vertex, BLUE, c, g);
    }
}

fn main() {
    // let points: Vec<Vector2> = vec![
    //     Vector2::new(0.4, 0.5),
    //     Vector2::new(0.6, 0.5),
    //     Vector2::new(0.5, 0.2),
    //     Vector2::new(0.5, 0.8),
    // ];

    // let points = vec![
    //     Vector2::new(0.8545189165195228, 0.03054690571899843),
    //     Vector2::new(0.7359666054926415, 0.09757792265981924),
    //     Vector2::new(0.20548556483395997, 0.11141869537040194),
    //     Vector2::new(0.01675129420586885, 0.12051964205677834),
    //     Vector2::new(0.2852457465283281, 0.149179106485832),
    //     Vector2::new(0.5700156839735175, 0.3305212298891148),
    //     Vector2::new(0.6327951274099004, 0.8253313276763707),
    //     Vector2::new(0.9272513099727112, 0.8712778711138446),
    //     Vector2::new(0.5454215894655622, 0.9233746637708448),
    //     Vector2::new(0.1870174524640723, 0.9633344884826402),
    // ];
    let mut points: Vec<Vector2> = vec![];
    let seed: &[u8; 32] = &[0; 32];
    let mut rng: StdRng = StdRng::from_seed(*seed);
    for _ in 0..100000 {
        points.push(Vector2::new(rng.gen(), rng.gen()));
    }

    let diagram = voronoi::generate_diagram(&points);

    let mut window: PistonWindow = WindowSettings::new("Voronoi", [WINDOW_WIDTH, WINDOW_HEIGHT])
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    window.set_lazy(true);
    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, g| {
            clear([0.0, 0.0, 0.0, 1.0], g);
            draw_voronoi_diagram(&diagram, c, g);
            draw_delauney_vertices(&diagram, c, g);
        });
    }
}
