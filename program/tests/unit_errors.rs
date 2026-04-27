/// Unit tests for error module — error code values and ProgramError conversion.
use project_incognitus::error::IncognitusError;

/// Verify each error variant maps to its expected u32 code.
#[test]
fn error_codes_match_expected_values() {
    assert_eq!(IncognitusError::InvalidDiscriminator as u32, 0x100);
    assert_eq!(IncognitusError::InvalidOwner as u32, 0x101);
    assert_eq!(IncognitusError::MissingRequiredSigner as u32, 0x102);
    assert_eq!(IncognitusError::AccountNotWritable as u32, 0x103);
    assert_eq!(IncognitusError::InvalidPda as u32, 0x104);
    assert_eq!(IncognitusError::AlreadyInitialized as u32, 0x105);
    assert_eq!(IncognitusError::NotInitialized as u32, 0x106);
    assert_eq!(IncognitusError::InvalidDataLength as u32, 0x107);
    assert_eq!(IncognitusError::CloseNotComplete as u32, 0x108);
    assert_eq!(IncognitusError::AlreadyClosing as u32, 0x109);
    assert_eq!(IncognitusError::InvalidSystemProgram as u32, 0x10A);
    assert_eq!(IncognitusError::InvalidRentSysvar as u32, 0x10B);
    assert_eq!(IncognitusError::ExceedsMaxAccountSize as u32, 0x10C);
    assert_eq!(IncognitusError::InvalidAllocateSize as u32, 0x10D);
    assert_eq!(IncognitusError::InstructionDataTooShort as u32, 0x10E);
    assert_eq!(IncognitusError::InvalidInstruction as u32, 0x10F);
    assert_eq!(IncognitusError::WrongNumberOfAccounts as u32, 0x110);
}

/// All error codes must be in the 0x100–0x1FF range.
#[test]
fn error_codes_in_custom_range() {
    let codes = [
        IncognitusError::InvalidDiscriminator as u32,
        IncognitusError::InvalidOwner as u32,
        IncognitusError::MissingRequiredSigner as u32,
        IncognitusError::AccountNotWritable as u32,
        IncognitusError::InvalidPda as u32,
        IncognitusError::AlreadyInitialized as u32,
        IncognitusError::NotInitialized as u32,
        IncognitusError::InvalidDataLength as u32,
        IncognitusError::CloseNotComplete as u32,
        IncognitusError::AlreadyClosing as u32,
        IncognitusError::InvalidSystemProgram as u32,
        IncognitusError::InvalidRentSysvar as u32,
        IncognitusError::ExceedsMaxAccountSize as u32,
        IncognitusError::InvalidAllocateSize as u32,
        IncognitusError::InstructionDataTooShort as u32,
        IncognitusError::InvalidInstruction as u32,
        IncognitusError::WrongNumberOfAccounts as u32,
    ];

    for code in codes {
        assert!(
            code >= 0x100 && code <= 0x1FF,
            "error code {:#06X} outside allowed range",
            code,
        );
    }
}

/// No duplicate error codes.
#[test]
fn error_codes_are_unique() {
    let codes = [
        IncognitusError::InvalidDiscriminator as u32,
        IncognitusError::InvalidOwner as u32,
        IncognitusError::MissingRequiredSigner as u32,
        IncognitusError::AccountNotWritable as u32,
        IncognitusError::InvalidPda as u32,
        IncognitusError::AlreadyInitialized as u32,
        IncognitusError::NotInitialized as u32,
        IncognitusError::InvalidDataLength as u32,
        IncognitusError::CloseNotComplete as u32,
        IncognitusError::AlreadyClosing as u32,
        IncognitusError::InvalidSystemProgram as u32,
        IncognitusError::InvalidRentSysvar as u32,
        IncognitusError::ExceedsMaxAccountSize as u32,
        IncognitusError::InvalidAllocateSize as u32,
        IncognitusError::InstructionDataTooShort as u32,
        IncognitusError::InvalidInstruction as u32,
        IncognitusError::WrongNumberOfAccounts as u32,
    ];

    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(
                codes[i], codes[j],
                "error codes at index {} and {} collide: {:#06X}",
                i, j, codes[i],
            );
        }
    }
}

/// ProgramError::Custom roundtrip — error converts to the expected Custom(u32).
#[test]
fn error_converts_to_program_error_custom() {
    use pinocchio::error::ProgramError;

    let pe: ProgramError = IncognitusError::InvalidPda.into();
    match pe {
        ProgramError::Custom(code) => assert_eq!(code, 0x104),
        other => panic!("expected ProgramError::Custom, got {:?}", other),
    }
}
