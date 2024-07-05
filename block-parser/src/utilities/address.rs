use super::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct AddressString<N: Network> {
    // The string representation of the address.
    string: String,
    _phantom: PhantomData<N>,
}

impl<N: Network> AddressString<N> {
    // Constructs a new `AddressString`.
    // Note that this method does not validate the address.
    pub fn new(string: String) -> Self {
        Self {
            string,
            _phantom: Default::default(),
        }
    }

    // Returns the string representation of the address.
    pub fn string(&self) -> &String {
        &self.string
    }

    // Returns an `Address` if the string is valid.
    pub fn address(&self) -> Result<Address<N>> {
        Address::from_str(&self.string)
    }
}

impl<N: Network> Debug for AddressString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.string())
    }
}

impl<N: Network> Display for AddressString<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.string())
    }
}

impl<N: Network> PartialEq for AddressString<N> {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

impl<N: Network> Eq for AddressString<N> {}

impl<N: Network> Hash for AddressString<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.string.hash(state)
    }
}
