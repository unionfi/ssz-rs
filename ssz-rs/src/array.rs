use crate::{
    de::{deserialize_homogeneous_composite, Deserialize, DeserializeError},
    error::{InstanceError, TypeError},
    lib::*,
    merkleization::{merkleize, pack, MerkleizationError, Merkleized, Node, BYTES_PER_CHUNK},
    ser::{serialize_composite, Serialize, SerializeError},
    SimpleSerialize, Sized,
};

impl<T, const N: usize> Sized for [T; N]
where
    T: SimpleSerialize,
{
    fn is_variable_size() -> bool {
        T::is_variable_size()
    }

    fn size_hint() -> usize {
        T::size_hint() * N
    }
}

impl<T, const N: usize> Serialize for [T; N]
where
    T: SimpleSerialize,
{
    fn serialize(&self, buffer: &mut Vec<u8>) -> Result<usize, SerializeError> {
        if N == 0 {
            return Err(TypeError::InvalidBound(N).into());
        }
        serialize_composite(self, buffer)
    }
}

impl<T, const N: usize> Deserialize for [T; N]
where
    T: SimpleSerialize,
{
    fn deserialize(encoding: &[u8]) -> Result<Self, DeserializeError> {
        if N == 0 {
            return Err(TypeError::InvalidBound(N).into());
        }

        if !T::is_variable_size() {
            let expected_length = N * T::size_hint();
            if encoding.len() < expected_length {
                return Err(DeserializeError::ExpectedFurtherInput {
                    provided: encoding.len(),
                    expected: expected_length,
                });
            }
            if encoding.len() > expected_length {
                return Err(DeserializeError::AdditionalInput {
                    provided: encoding.len(),
                    expected: expected_length,
                });
            }
        }
        let elements = deserialize_homogeneous_composite(encoding)?;
        elements.try_into().map_err(|elements: Vec<T>| {
            InstanceError::Exact { required: N, provided: elements.len() }.into()
        })
    }
}

impl<T, const N: usize> Merkleized for [T; N]
where
    T: SimpleSerialize,
{
    fn hash_tree_root(&mut self) -> Result<Node, MerkleizationError> {
        if T::is_composite_type() {
            let mut chunks = vec![0u8; self.len() * BYTES_PER_CHUNK];
            for (i, elem) in self.iter_mut().enumerate() {
                let chunk = elem.hash_tree_root()?;
                let range = i * BYTES_PER_CHUNK..(i + 1) * BYTES_PER_CHUNK;
                chunks[range].copy_from_slice(chunk.as_ref());
            }
            merkleize(&chunks, None)
        } else {
            let chunks = pack(self)?;
            merkleize(&chunks, None)
        }
    }
}

impl<T, const N: usize> SimpleSerialize for [T; N]
where
    T: SimpleSerialize,
{
    fn is_composite_type() -> bool {
        T::is_composite_type()
    }
}
