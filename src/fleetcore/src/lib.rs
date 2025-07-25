//! Shared types for zero-knowledge  input/output,
//! used for communication between the host, guest (zkVM), and verifier.

use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};

// -----------------------------------------------------------------------------
// INPUT STRUCTS
// -----------------------------------------------------------------------------

/// Input to zkVM programs sent by the rust code for input on the methods join, wave and win
/// The struct is read by the zkvm code and the data is used to generate the output Journal
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BaseInputs {
    pub gameid: String,
    pub fleet: String,
    pub board: Vec<u8>,
    pub random: String,
    pub token_auth: Option<TokenAuth>,
}

/// Input to zkVM programs sent by the rust code for input on the methods fire and report
/// The struct is read by the zkvm code and the data is used to generate the output Journal
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct FireInputs {
    pub gameid: String,
    pub fleet: String,
    pub board: Vec<u8>,
    pub random: String,
    pub target: String,
    pub pos: u8,
    pub token_auth: Option<TokenAuth>,
}

/// Struct that contains the necessary fields to prove token ownership
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TokenAuth {
    pub token: Vec<u8>,        // Decrypted token
    pub expected_hash: Digest, // Hash previously committed
}

// -----------------------------------------------------------------------------
// NETWORK COMMUNICATION
// -----------------------------------------------------------------------------

/// Enum used to define the command that will be sent to the server by the host in the communication packet
#[derive(Deserialize, Serialize)]
pub enum Command {
    Join,
    Fire,
    Report,
    Wave,
    Win,
    Contest,
}

/// Struct used to specify the packet sent from the client to the blockchain server
#[derive(Deserialize, Serialize)]
pub struct CommunicationData {
    pub cmd: Command,
    pub receipt: Receipt,
    pub token_data: Option<EncryptedToken>,
}

#[derive(Deserialize, Serialize)]
pub struct EncryptedToken {
    pub enc_token: String,
    pub token_hash: Digest,
    pub pub_rsa_key: Vec<u8>,
}

/// Wrapper for signed messages.
#[derive(Serialize, Deserialize)]
pub struct SignedMessage<T> {
    pub payload: T,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

// -----------------------------------------------------------------------------
// JOURNALS
// -----------------------------------------------------------------------------

/// Struct used to specify the  output journal for join, wave and win methods
#[derive(Deserialize, PartialEq, Eq, Serialize, Default)]
pub struct BaseJournal {
    pub gameid: String,
    pub fleet: String,
    pub board: Digest,
    pub token_commitment: Digest,
}

/// Struct used to specify the output journal for fire method
#[derive(Deserialize, PartialEq, Eq, Serialize, Default)]
pub struct FireJournal {
    pub gameid: String,
    pub fleet: String,
    pub board: Digest,
    pub target: String,
    pub pos: u8,
    pub token_commitment: Digest,
}

/// Struct used to specify the output journal for report method
#[derive(Deserialize, PartialEq, Eq, Serialize, Default)]
pub struct ReportJournal {
    pub gameid: String,
    pub fleet: String,
    pub report: String,
    pub pos: u8,
    pub board: Digest,
    pub next_board: Digest,
    pub token_commitment: Digest,
}
