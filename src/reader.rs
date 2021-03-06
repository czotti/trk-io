use std::{
    fs::File,
    io::{BufReader, Result},
    path::Path,
};

use byteorder::{BigEndian, ByteOrder, LittleEndian, ReadBytesExt};

use crate::{
    cheader::Endianness,
    tractogram::{Point, Points, Streamlines, Tractogram, TractogramItem},
    Affine, ArraySequence, Header, Translation,
};

pub struct Reader {
    reader: BufReader<File>,
    endianness: Endianness,
    pub header: Header,
    pub affine_to_rasmm: Affine,
    pub translation: Translation,

    nb_scalars: usize,
    nb_properties: usize,
    nb_floats_per_point: usize,
    float_buffer: Vec<f32>,
}

impl Reader {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Reader> {
        let mut reader = BufReader::new(File::open(&path)?);
        let (header, endianness) = Header::read(&mut reader)?;
        let affine_to_rasmm = header.affine_to_rasmm;
        let translation = header.translation;
        let nb_scalars = header.scalars_name.len();
        let nb_properties = header.properties_name.len();
        let nb_floats_per_point = 3 + nb_scalars;

        Ok(Reader {
            reader,
            endianness,
            header,
            affine_to_rasmm,
            translation,
            nb_scalars,
            nb_properties,
            nb_floats_per_point,
            float_buffer: Vec::with_capacity(300),
        })
    }

    pub fn read_all(&mut self) -> Tractogram {
        match self.endianness {
            Endianness::Little => self.read_all_::<LittleEndian>(),
            Endianness::Big => self.read_all_::<BigEndian>(),
        }
    }

    fn read_all_<E: ByteOrder>(&mut self) -> Tractogram {
        // TODO Anything we can do to reerve?
        let mut lengths = Vec::new();
        let mut v = Vec::with_capacity(300);
        let mut scalars = ArraySequence::with_capacity(300);
        let mut properties = ArraySequence::with_capacity(300);
        while let Ok(nb_points) = self.reader.read_i32::<E>() {
            lengths.push(nb_points as usize);
            self.read_streamline::<E>(&mut v, &mut scalars, nb_points as usize);
            self.read_properties_to_arr::<E>(&mut properties);
        }

        self.float_buffer = vec![];
        Tractogram::new(Streamlines::new(lengths, v), scalars, properties)
    }

    fn read_streamline<E: ByteOrder>(
        &mut self,
        points: &mut Points,
        scalars: &mut ArraySequence<f32>,
        nb_points: usize,
    ) {
        // Vec::resize never decreases capacity, it can only increase it
        // so there won't be any useless allocation.
        let nb_floats = nb_points * self.nb_floats_per_point;
        self.float_buffer.resize(nb_floats as usize, 0.0);
        self.reader.read_f32_into::<E>(self.float_buffer.as_mut_slice()).unwrap();

        for floats in self.float_buffer.chunks(self.nb_floats_per_point) {
            let p = Point::new(floats[0], floats[1], floats[2]);
            points.push((self.affine_to_rasmm * p) + self.translation);

            for f in &floats[3..] {
                scalars.push(*f);
            }
        }
        scalars.end_push();
    }

    fn read_properties_to_arr<E: ByteOrder>(&mut self, properties: &mut ArraySequence<f32>) {
        for _ in 0..self.nb_properties {
            properties.push(self.reader.read_f32::<E>().unwrap());
        }
        properties.end_push();
    }

    fn read_properties_to_vec<E: ByteOrder>(&mut self, properties: &mut Vec<f32>) {
        for _ in 0..self.nb_properties {
            properties.push(self.reader.read_f32::<E>().unwrap());
        }
    }
}

impl Iterator for Reader {
    type Item = TractogramItem;

    fn next(&mut self) -> Option<TractogramItem> {
        if let Ok(nb_points) = match self.endianness {
            Endianness::Little => self.reader.read_i32::<LittleEndian>(),
            Endianness::Big => self.reader.read_i32::<BigEndian>(),
        } {
            let nb_points = nb_points as usize;
            let mut streamline = Vec::with_capacity(nb_points);
            let mut scalars = ArraySequence::with_capacity(nb_points * self.nb_scalars);
            let mut properties = Vec::with_capacity(self.nb_properties);
            match self.endianness {
                Endianness::Little => {
                    self.read_streamline::<LittleEndian>(&mut streamline, &mut scalars, nb_points);
                    self.read_properties_to_vec::<LittleEndian>(&mut properties);
                }
                Endianness::Big => {
                    self.read_streamline::<BigEndian>(&mut streamline, &mut scalars, nb_points);
                    self.read_properties_to_vec::<BigEndian>(&mut properties);
                }
            };

            Some((streamline, scalars, properties))
        } else {
            None
        }
    }
}
