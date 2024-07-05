use super::*;

pub type BondedMapping<N> = HashMap<AddressBytes<N>, (AddressBytes<N>, u64)>;
pub type UnbondedMapping<N> = HashMap<AddressBytes<N>, (u64, u32)>;
pub type WithdrawMapping<N> = HashMap<AddressBytes<N>, AddressBytes<N>>;

// Decodes the block from little-endian archive.
pub fn decode_block<N: Network, R: Read>(mut reader: R) -> Result<Block<N>> {
    Ok(Block::read_le(&mut reader)?)
}

// Decodes the bonded mapping from little-endian archive.
pub fn decode_bonded_mapping<N: Network, R: Read>(mut reader: R) -> Result<BondedMapping<N>> {
    // Get the length of the bonded mapping.
    let length = u32::read_le(&mut reader)?;
    // Initialize the bonded mapping.
    let mut bonded = BondedMapping::with_capacity(length as usize);
    // Decode the key-value pairs.
    for i in 0..length {
        println!("Entry: {}", i);
        let key = get_address_bytes_from_plaintext(&mut reader)?;
        let value = {
            // Read the struct member archive from the value archive.
            let mut members = get_struct_member_bytes_from_value::<N, _>(&mut reader, 2)?;
            // Check that there are 2 members.
            ensure!(members.len() == 2, "Expected 2 members");
            // Get the amount.
            // Note that this unwrap is safe because we have checked that there are 2 members.
            let amount = u64::read_le(members.pop().unwrap().as_slice())?;
            // Get the address archive.
            // Note that this unwrap is safe because we have checked that there are 2 members.
            let address_bytes = AddressBytes::new(members.pop().unwrap());

            (address_bytes, amount)
        };
        bonded.insert(key, value);
    }
    Ok(bonded)
}

// Decodes the unbonding mapping from little-endian archive.
pub fn decode_unbonding_mapping<N: Network, R: Read>(mut reader: R) -> Result<UnbondedMapping<N>> {
    // Get the length of the unbonding mapping.
    let length = u32::read_le(&mut reader)?;
    // Initialize the unbonding mapping.
    let mut unbonding = UnbondedMapping::with_capacity(length as usize);
    // Decode the key-value pairs.
    for _ in 0..length {
        let key = get_address_bytes_from_plaintext(&mut reader)?;
        let value = {
            // Read the struct member archive from the value archive.
            let members = get_struct_member_bytes_from_value::<N, _>(&mut reader, 2)?;
            // Get the amount.
            let amount = u64::read_le(members[0].as_slice())?;
            // Get the block height.
            let block_height = u32::read_le(members[1].as_slice())?;
            (amount, block_height)
        };
        unbonding.insert(key, value);
    }
    Ok(unbonding)
}

// Decodes the withdraw mapping from little-endian archive.
pub fn decode_withdraw_mapping<N: Network, R: Read>(mut reader: R) -> Result<WithdrawMapping<N>> {
    // Get the length of the withdraw mapping.
    let length = u32::read_le(&mut reader)?;
    // Initialize the withdraw mapping.
    let mut withdraw = WithdrawMapping::with_capacity(length as usize);
    // Decode the key-value pairs.
    for _ in 0..length {
        let key = get_address_bytes_from_plaintext(&mut reader)?;
        let value = get_address_bytes_from_value(&mut reader)?;
        withdraw.insert(key, value);
    }
    Ok(withdraw)
}

// A helper function to check and get the struct member plaintext archive from value archive.
fn get_struct_member_bytes_from_value<N: Network, R: Read>(
    mut reader: R,
    num_members: u8,
) -> Result<Vec<Vec<u8>>> {
    // Initialize the struct member archive.
    let mut members = Vec::with_capacity(num_members as usize);
    // Get the variant of the value.
    let variant = u8::read_le(&mut reader).unwrap();
    // Check that it is a `Plaintext` variant.
    ensure!(variant == 0, "Expected a `Plaintext` variant");
    // Check that it is a `Struct` variant
    let variant = u8::read_le(&mut reader).unwrap();
    ensure!(variant == 1, "Expected a `Struct` variant");
    // Get the number of members in the struct.
    let num_members = u8::read_le(&mut reader).unwrap();
    println!("Num members: {}", num_members);
    // Get the members.
    for i in 0..num_members {
        println!("Member: {}", i);
        // Read the identifier.
        let id = Identifier::<N>::read_le(&mut reader).unwrap();
        println!("id: {:?}", id);
        // Read the plaintext value (in 2 steps to prevent infinite recursion).
        let num_bytes = u16::read_le(&mut reader)?;
        println!("num_bytes: {:?}", num_bytes);
        // Read the plaintext archive.
        let mut bytes = Vec::new();
        (&mut reader).take(num_bytes as u64).read_to_end(&mut bytes)?;
        println!("archive: {:?}", bytes);
        // Add the member.
        members.push(bytes);
    }
    Ok(members)
}

// A helper function to check and get the address archive from value archive.
fn get_address_bytes_from_value<N: Network, R: Read>(mut reader: R) -> Result<AddressBytes<N>> {
    // Get the variant of the address.
    let variant = u8::read_le(&mut reader)?;
    // Check that it is a `Plaintext` variant.
    ensure!(variant == 0, "Expected a `Plaintext` variant");
    // Check and get the address archive.
    get_address_bytes_from_plaintext(&mut reader)
}

// A helper function to check and get the address archive from plaintext archive.
fn get_address_bytes_from_plaintext<N: Network, R: Read>(mut reader: R) -> Result<AddressBytes<N>> {
    // Get the variant of the address.
    let variant = u8::read_le(&mut reader)?;
    // Check that it is a `Literal` variant.
    ensure!(variant == 0, "Expected a `Literal` variant");
    // Get the literal variant of the address.
    let literal_variant = u16::read_le(&mut reader).unwrap();
    // Check that it is a `Address` variant.
    ensure!(literal_variant == 0, "Expected a `Address` variant");
    // Get the address archive.
    let address_bytes = AddressBytes::read_le(&mut reader).unwrap();
    Ok(address_bytes)
}
