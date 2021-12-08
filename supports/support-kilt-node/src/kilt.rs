use codec::{Decode, Encode};
use scale_info::TypeInfo;

#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd, TypeInfo)]
pub enum DidEncryptionKey {
	/// An X25519 public key.
	X25519([u8; 32]),
}
