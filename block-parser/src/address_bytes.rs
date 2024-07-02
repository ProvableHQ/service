use std::fmt::{Debug, Display};
use super::*;
use snarkvm::prelude::ToBytes;
use std::hash::Hash;

#[derive(Clone)]
pub struct AddressBytes<N: Network> {
    // The little-endian bytes of the address.
    // This is approximately 32 bytes, which implies that clones are not prohibitively expensive.
    // For further performance improvements, consider using `smallvec`.
    bytes_le: Vec<u8>,
    address: OnceCell<Option<Address<N>>>,
}

impl<N: Network> AddressBytes<N> {
    pub fn new(bytes_le: Vec<u8>) -> Self {
        Self {
            bytes_le,
            address: OnceCell::new(),
        }
    }

    pub fn from_address(address: Address<N>) -> Result<Self> {
        let bytes_le = address.to_bytes_le()?;
        Ok(Self::new(bytes_le))
    }

    pub fn bytes_le(&self) -> &[u8] {
        &self.bytes_le
    }

    pub fn address(&self) -> &Option<Address<N>> {
        self.address
            .get_or_init(|| Address::from_bytes_le(&self.bytes_le).ok())
    }
}

impl<N: Network> FromBytes for AddressBytes<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self>
    where
        Self: Sized,
    {
        let mut bytes_le = vec![0; Address::<N>::size_in_bytes()];
        reader.read_exact(&mut bytes_le)?;
        Ok(AddressBytes::new(bytes_le))
    }
}

impl<N: Network> Debug for AddressBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.address())
    }
}

impl<N: Network> Display for AddressBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.address())
    }
}

impl<N: Network> PartialEq for AddressBytes<N> {
    fn eq(&self, other: &Self) -> bool {
        self.bytes_le == other.bytes_le
    }
}

impl<N: Network> Eq for AddressBytes<N> {}

impl<N: Network> Hash for AddressBytes<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bytes_le.hash(state)
    }
}
