use pinocchio::error::ProgramError;

/// Custom program errors.
///
/// Range: 0x100 — 0x1FF for account management errors.
#[repr(u32)]
pub enum IncognitusError {
    // -- Account validation errors --
    /// Account discriminator mismatch.
    InvalidDiscriminator = 0x100,

    /// Account is not owned by this program.
    InvalidOwner = 0x101,

    /// Account is not a signer.
    MissingRequiredSigner = 0x102,

    /// Account is not writable.
    AccountNotWritable = 0x103,

    /// PDA derivation mismatch.
    InvalidPda = 0x104,

    /// Account already initialized (discriminator is not zeroed).
    AlreadyInitialized = 0x105,

    /// Account not initialized (discriminator is zeroed).
    NotInitialized = 0x106,

    /// Invalid account data length.
    InvalidDataLength = 0x107,

    // -- Close / delete errors --
    /// Close operation: account has non-zero close index remaining.
    CloseNotComplete = 0x108,

    /// Close operation: closing discriminator already set.
    AlreadyClosing = 0x109,

    /// Provided system program is incorrect.
    InvalidSystemProgram = 0x10A,

    /// Provided rent sysvar is incorrect.
    InvalidRentSysvar = 0x10B,

    /// Allocate: new size exceeds maximum (10 MB).
    ExceedsMaxAccountSize = 0x10C,

    /// Allocate: new size is not aligned to 10kb increment.
    InvalidAllocateSize = 0x10D,

    /// Instruction data too short.
    InstructionDataTooShort = 0x10E,

    /// Invalid instruction discriminator.
    InvalidInstruction = 0x10F,

    /// Wrong number of accounts provided.
    WrongNumberOfAccounts = 0x110,
}

impl From<IncognitusError> for ProgramError {
    fn from(e: IncognitusError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
