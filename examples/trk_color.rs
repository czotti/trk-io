extern crate docopt;
extern crate trk_io;

use std::path::Path;
use std::str;

use docopt::Docopt;
use trk_io::{Header, Point, Reader, Writer};

static USAGE: &'static str = "
Color a TrackVis (.trk) file

Usage:
  trk_color uniform <r> <g> <b> <input> <output> [options]
  trk_color orientation <input> <output> [options]
  trk_color (-h | --help)
  trk_color (-v | --version)

Options:
  -h --help     Show this screen.
  -v --version  Show version.
";

fn main() {
    let version = String::from(env!("CARGO_PKG_VERSION"));
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());

    let input = Path::new(args.get_str("<input>"));
    if !input.exists() {
        panic!("Input trk '{:?}' doesn't exist.", input);
    }

    let reader = Reader::new(args.get_str("<input>")).expect("Read header");
    let mut header = reader.header.clone();
    header.add_scalar("color_x").unwrap();
    header.add_scalar("color_y").unwrap();
    header.add_scalar("color_z").unwrap();

    if args.get_bool("uniform") {
        let r = args.get_str("<r>").parse::<f32>().unwrap();
        let g = args.get_str("<g>").parse::<f32>().unwrap();
        let b = args.get_str("<b>").parse::<f32>().unwrap();
        uniform(reader, header, args.get_str("<output>"), r, g, b);
    } else if args.get_bool("orientation") {
        orientation(reader, header, args.get_str("<output>"));
    }
}

fn uniform(reader: Reader, header: Header, write_to: &str, r: f32, g: f32, b: f32) {
    let mut writer = Writer::new(write_to, Some(header)).unwrap();
    for (streamline, mut scalars, properties) in reader.into_iter() {
        scalars.push(vec![r; streamline.len()]);
        scalars.push(vec![g; streamline.len()]);
        scalars.push(vec![b; streamline.len()]);

        writer.write((streamline, scalars, properties));
    }
}

fn orientation(reader: Reader, header: Header, write_to: &str) {
    let mut writer = Writer::new(write_to, Some(header)).unwrap();
    for (streamline, mut scalars, properties) in reader.into_iter() {
        let mut r = Vec::with_capacity(streamline.len());
        let mut g = Vec::with_capacity(streamline.len());
        let mut b = Vec::with_capacity(streamline.len());

        // Scope to avoid r, g, b mutable sharing
        {
            let mut add = |p1: &Point, p2: &Point| {
                let x = p2.x - p1.x;
                let y = p2.y - p1.y;
                let z = p2.z - p1.z;
                let norm = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
                r.push((x / norm).abs() * 255.0);
                g.push((y / norm).abs() * 255.0);
                b.push((z / norm).abs() * 255.0);
            };

            // Manage first point
            add(&streamline[1], &streamline[0]);

            for p in streamline.windows(3) {
                add(&p[2], &p[0]);
            }

            // Manage last point
            add(&streamline[streamline.len() - 2], &streamline[streamline.len() - 1]);
        }

        scalars.push(r);
        scalars.push(g);
        scalars.push(b);
        writer.write((streamline, scalars, properties));
    }
}
