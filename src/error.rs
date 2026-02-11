use core::convert::TryFrom;

use pinocchio::error::{ProgramError, ToStr};

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniPinoNftErr {
    PdaErr = 1000,
    UninitPda = 1001,
    ReInitPda = 1002,
}

impl TryFrom<u32> for UniPinoNftErr {
    type Error = ProgramError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1000 => Ok(Self::PdaErr),
            1001 => Ok(Self::UninitPda),
            1002 => Ok(Self::ReInitPda),
            _ => Err(ProgramError::InvalidArgument),
        }
    }
}

impl ToStr for UniPinoNftErr {
    fn to_str(&self) -> &'static str {
        match self {
            Self::PdaErr => "PDA derivation failed",
            Self::UninitPda => "PDA is uninitialized",
            Self::ReInitPda => "PDA already initialized",
        }
    }
}

impl From<UniPinoNftErr> for ProgramError {
    fn from(e: UniPinoNftErr) -> Self {
        ProgramError::Custom(e as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_are_stable() {
        assert_eq!(UniPinoNftErr::PdaErr as u32, 1000);
        assert_eq!(UniPinoNftErr::UninitPda as u32, 1001);
        assert_eq!(UniPinoNftErr::ReInitPda as u32, 1002);
    }

    #[test]
    fn test_error_to_str_is_static() {
        assert_eq!(UniPinoNftErr::PdaErr.to_str(), "PDA derivation failed");
        assert_eq!(UniPinoNftErr::UninitPda.to_str(), "PDA is uninitialized");
        assert_eq!(UniPinoNftErr::ReInitPda.to_str(), "PDA already initialized");
    }

    #[test]
    fn test_error_converts_to_program_error_custom() {
        let err: ProgramError = UniPinoNftErr::UninitPda.into();
        assert_eq!(err, ProgramError::Custom(1001));
    }
}
