
extern crate trk_io;

use trk_io::{Affine, ArraySequence, Header, Point, Properties, Reader, Scalars, Translation};

#[test]
fn test_load_empty() {
    let (streamlines, scalars, properties) = Reader::new("data/empty.trk").unwrap().read_all();
    assert_eq!(streamlines.len(), 0);
    for _ in &streamlines {
        panic!("Failed test.");
    }
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    // Test generator
    let reader = Reader::new("data/empty.trk").unwrap();
    for _ in reader.into_iter() {
        panic!("Failed test.");
    }
}

#[test]
fn test_load_simple() {
    let first = [Point::new(0.0, 1.0, 2.0)];
    let second = [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
    let third = [Point::new(0.0, 1.0, 2.0),
                 Point::new(3.0, 4.0, 5.0),
                 Point::new(6.0, 7.0, 8.0),
                 Point::new(9.0, 10.0, 11.0),
                 Point::new(12.0, 13.0, 14.0)];

    let (streamlines, scalars, properties) = Reader::new("data/simple.trk").unwrap().read_all();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    // Test generator
    let reader = Reader::new("data/empty.trk").unwrap();
    for (i, streamline) in reader.into_iter().enumerate() {
        if i == 0 {
            assert_eq!(streamline, first);
        } else if i == 1 {
            assert_eq!(streamline, second);
        } else if i == 2 {
            assert_eq!(streamline, third);
        } else {
            panic!("Failed test.");
        }
    }
}

#[test]
fn test_load_standard() {
    let mut reader = Reader::new("data/standard.trk").unwrap();
    let (streamlines, scalars, properties) = reader.read_all();
    assert_eq!(reader.affine, Affine::new(1.0, 0.0, 0.0,
                                          0.0, 1.0, 0.0,
                                          0.0, 0.0, 1.0));
    assert_eq!(reader.translation, Translation::new(-0.5, -1.5, -1.0));

    assert_eq!(streamlines.len(), 120);
    assert_eq!(streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, 1.5, 3.0)]);
    assert_eq!(streamlines[1], [Point::new(-0.5, 1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, -1.5, 3.0)]);
    assert!(scalars.is_empty());
    assert!(properties.is_empty());

    // Test generator
    let reader = Reader::new("data/empty.trk").unwrap();
    for streamline in reader.into_iter() {
        assert_eq!(streamline.len(), 3);
    }
}

#[test]
fn test_load_standard_lps() {
    let mut reader = Reader::new("data/standard.LPS.trk").unwrap();
    let (streamlines, scalars, properties) = reader.read_all();
    assert_eq!(reader.affine, Affine::new(-1.0, 0.0, 0.0,
                                          0.0, -1.0, 0.0,
                                          0.0, 0.0, 1.0));
    assert_eq!(reader.translation, Translation::new(3.5, 13.5, -1.0));

    assert_eq!(streamlines.len(), 120);
    assert_eq!(streamlines[0], [Point::new(-0.5, -1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, 1.5, 3.0)]);
    assert_eq!(streamlines[1], [Point::new(-0.5, 1.5, 1.0),
                                Point::new(0.0, 0.0, 2.0),
                                Point::new(0.5, -1.5, 3.0)]);
    assert!(scalars.is_empty());
    assert!(properties.is_empty());
}

#[test]
fn test_load_complex() {
    let mut reader = Reader::new("data/complex.trk").unwrap();
    let (streamlines, scalars, properties) = reader.read_all();
    assert_eq!(reader.affine, Affine::new(1.0, 0.0, 0.0,
                                          0.0, 1.0, 0.0,
                                          0.0, 0.0, 1.0));
    assert_eq!(reader.translation, Translation::new(-0.5, -0.5, -0.5));

    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], [Point::new(0.0, 1.0, 2.0)]);
    assert_eq!(streamlines[1], [Point::new(0.0, 1.0, 2.0),
                                Point::new(3.0, 4.0, 5.0)]);
    assert_eq!(streamlines[2], [Point::new(0.0, 1.0, 2.0),
                                Point::new(3.0, 4.0, 5.0),
                                Point::new(6.0, 7.0, 8.0),
                                Point::new(9.0, 10.0, 11.0),
                                Point::new(12.0, 13.0, 14.0)]);

    check_complex_scalars_and_properties(reader.header, scalars, properties);
}

#[test]
fn test_load_complex_big_endian() {
    let first = [Point::new(0.0, 1.0, 2.0)];
    let second = [Point::new(0.0, 1.0, 2.0), Point::new(3.0, 4.0, 5.0)];
    let third = [Point::new(0.0, 1.0, 2.0),
                 Point::new(3.0, 4.0, 5.0),
                 Point::new(6.0, 7.0, 8.0),
                 Point::new(9.0, 10.0, 11.0),
                 Point::new(12.0, 13.0, 14.0)];

    let mut reader = Reader::new("data/complex_big_endian.trk").unwrap();
    let (streamlines, scalars, properties) = reader.read_all();
    assert_eq!(streamlines.len(), 3);
    assert_eq!(streamlines[0], first);
    assert_eq!(streamlines[1], second);
    assert_eq!(streamlines[2], third);
    check_complex_scalars_and_properties(reader.header, scalars, properties);

    // Test generator
    let reader = Reader::new("data/complex_big_endian.trk").unwrap();
    for (i, streamline) in reader.into_iter().enumerate() {
        if i == 0 {
            assert_eq!(streamline, first);
        } else if i == 1 {
            assert_eq!(streamline, second);
        } else if i == 2 {
            assert_eq!(streamline, third);
        } else {
            panic!("Failed test.");
        }
    }
}

fn check_complex_scalars_and_properties(
    header: Header,
    scalars: Vec<Scalars>,
    properties: Vec<Properties>
) {
    // Scalars
    assert_eq!(header.scalars_name, vec![
        String::from("colors"),
        String::from("colors"),
        String::from("colors"),
        String::from("fa")]);
    assert!(scalars[0] == ArraySequence::new(
        vec![1, 2, 5], vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
    assert!(scalars[1] == ArraySequence::new(
        vec![1, 2, 5], vec![0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0]));
    assert!(scalars[2] == ArraySequence::new(
        vec![1, 2, 5], vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0]));
    assert!(scalars[3] == ArraySequence::new(
        vec![1, 2, 5],
        vec![0.200000003, 0.300000012, 0.400000006, 0.500000000,
             0.600000024, 0.600000024, 0.699999988, 0.800000012]));

    // Properties
    assert_eq!(header.properties_name, vec![
        String::from("mean_colors"),
        String::from("mean_colors"),
        String::from("mean_colors"),
        String::from("mean_curvature"),
        String::from("mean_torsion")]);
    assert_eq!(properties[0], vec![1.0, 0.0, 0.0]);
    assert_eq!(properties[1], vec![0.0, 1.0, 0.0]);
    assert_eq!(properties[2], vec![0.0, 0.0, 1.0]);
    assert_eq!(properties[3], vec![1.11000001, 2.11000001, 3.11000001]);
    assert_eq!(properties[4], vec![1.22000003, 2.22000003, 3.22000003]);
}
