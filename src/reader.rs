use std::fs::File;
use std::io::{BufReader, Result};
use std::path::Path;

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};

use {Affine, ArraySequence, Header, Point, Points, Properties, Scalars, Streamlines, Translation};
use cheader::{Endianness};

pub struct Reader {
    reader: BufReader<File>,
    endianness: Endianness,
    pub header: Header,
    pub affine: Affine,
    pub translation: Translation,

    nb_floats_per_point: usize,
    float_buffer: Vec<f32>
}

impl Reader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Reader> {
        let f = File::open(path).expect("Can't read trk file.");
        let mut reader = BufReader::new(f);

        let (header, endianness) = Header::read(&mut reader)?;
        let affine = header.affine;
        let translation = header.translation;
        let nb_floats_per_point = 3 + header.scalars_name.len() as usize;

        Ok(Reader {
            reader, endianness, header, affine, translation,
            nb_floats_per_point, float_buffer: Vec::with_capacity(300)
        })
    }

    pub fn read_all(&mut self) -> (Streamlines, Vec<Scalars>, Vec<Properties>) {
        match self.endianness {
            Endianness::Little => self.read_all_::<LittleEndian>(),
            Endianness::Big => self.read_all_::<BigEndian>()
        }
    }

    fn read_all_<E: ByteOrder>(&mut self) -> (Streamlines, Vec<Scalars>, Vec<Properties>) {
        let mut lengths = Vec::new();
        let mut v = Vec::with_capacity(300);
        let (mut scalars, mut properties) = self.get_sp();
        while let Ok(nb_points) = self.reader.read_i32::<E>() {
            lengths.push(nb_points as usize);
            self.read_streamline::<E>(
                &mut v, &mut scalars, &mut properties, nb_points as usize);
        }
        self.float_buffer = vec![];
        (Streamlines::new(lengths, v), scalars, properties)
    }

    fn read_streamline<E: ByteOrder>(
        &mut self,
        points: &mut Points,
        scalars: &mut Vec<Scalars>,
        properties: &mut Vec<Properties>,
        nb_points: usize)
    {
        // Vec::resize never decreases capacity, it can only increase it
        // so there won't be any useless allocation.
        let nb_floats = nb_points * self.nb_floats_per_point;
        self.float_buffer.resize(nb_floats as usize, 0.0);
        self.reader.read_f32_into::<E>(
            self.float_buffer.as_mut_slice()).unwrap();

        for floats in self.float_buffer.chunks(self.nb_floats_per_point) {
            let p = Point::new(floats[0], floats[1], floats[2]);
            points.push((self.affine * p) + self.translation);

            for (scalar, f) in scalars.iter_mut().zip(&floats[3..]) {
                scalar.push(*f);
            }
        }

        for scalar in scalars.iter_mut() {
            scalar.end_push();
        }

        for property in properties.iter_mut() {
            property.push(self.reader.read_f32::<E>().unwrap());
        }
    }

    fn get_sp(&mut self) -> (Vec<Scalars>, Vec<Properties>) {
        let scalars = vec![ArraySequence::empty(); self.header.scalars_name.len()];
        let properties = vec![vec![]; self.header.properties_name.len()];
        (scalars, properties)
    }
}

impl Iterator for Reader {
    type Item = Points;

    fn next(&mut self) -> Option<Points> {
        if let Ok(nb_points) = match self.endianness {
            Endianness::Little => self.reader.read_i32::<LittleEndian>(),
            Endianness::Big => self.reader.read_i32::<BigEndian>()
        } {
            let mut points = Vec::with_capacity(nb_points as usize);
            let (mut scalars, mut properties) = self.get_sp();
            match self.endianness {
                Endianness::Little => self.read_streamline::<LittleEndian>(
                    &mut points, &mut scalars, &mut properties, nb_points as usize),
                Endianness::Big => self.read_streamline::<BigEndian>(
                    &mut points, &mut scalars, &mut properties, nb_points as usize)
            };
            Some(points)
        } else {
            None
        }
    }
}
