/// This trait allows for serialisation of little endian data types to a vector of bytes.
pub trait LeBytesSerialisable {
    /// Serialise the data type to a vector of bytes in little endian format.
    fn to_le_vec(&self) -> Vec<u8>;

    /// Deserialise the data type from a vector of bytes in little endian format.
    fn from_le_vec(bytes: Vec<u8>) -> Self;
}

// Array serialisation.
impl<T: LeBytesSerialisable> LeBytesSerialisable for Vec<T> {
    fn to_le_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for item in self {
            bytes.extend(item.to_le_vec());
        }

        bytes
    }

    fn from_le_vec(bytes: Vec<u8>) -> Self {
        let mut items = Vec::new();
        let mut chunks = bytes.chunks_exact(T::to_le_vec(&T::from_le_vec(vec![0])).len());

        for chunk in chunks.by_ref() {
            items.push(T::from_le_vec(chunk.to_vec()));
        }

        items
    }
}

// String serialisation.
impl LeBytesSerialisable for String {
    fn to_le_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn from_le_vec(bytes: Vec<u8>) -> Self {
        String::from_utf8(bytes).unwrap()
    }
}

// Boolean serialisation.
impl LeBytesSerialisable for bool {
    fn to_le_vec(&self) -> Vec<u8> {
        vec![u8::from(*self)]
    }

    fn from_le_vec(bytes: Vec<u8>) -> Self {
        bytes[0] != 0
    }
}

// Automatic serialisation of types that implement to_le_bytes and from_le_bytes.
macro_rules! auto_serialise_le_bytes {
    ($($t:ty),*) => {
        $(
            impl LeBytesSerialisable for $t {
                fn to_le_vec(&self) -> Vec<u8> {
                    self.to_le_bytes().to_vec()
                }

                fn from_le_vec(bytes: Vec<u8>) -> Self {
                    <$t>::from_le_bytes(bytes.try_into().unwrap())
                }
            }
        )*
    };
}

auto_serialise_le_bytes!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, f32, f64);
