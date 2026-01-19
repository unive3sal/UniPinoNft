use num_derive::FromPrimitive;
use pinocchio::program_error::{ProgramError, ToStr};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Error, FromPrimitive)]
pub enum UniPinoNftErr {
    #[error("Fail to find a valid PDA")]
    PdaErr,
    #[error("Instruction try to access uninit PDA")]
    UninitPda,
    #[error("Instruction try to re-init exist PDA")]
    ReInitPda,
}

impl ToStr for UniPinoNftErr {
    fn to_str<E>(&self) -> &'static str
    where
        E: 'static + ToStr + TryFrom<u32>,
    {
        match self {
            Self::PdaErr => "ERROR: Fail to find a valid PDA",
            Self::UninitPda => "ERROR: Instruction try to access uninit PDA",
            Self::ReInitPda => "ERROR: Instruction try to re-init exist PDA",
        }
    }
}

impl From<UniPinoNftErr> for ProgramError {
    fn from(e: UniPinoNftErr) -> Self {
        ProgramError::Custom(e as u32)
    }
}
