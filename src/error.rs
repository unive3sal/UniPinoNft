use pinocchio::program_error::ProgramError;

pub enum UniPinoNftErr {
    PdaErr,
    ReInitPda,
}

impl pinocchio::program_error::ToStr for UniPinoNftErr {
    fn to_str<E>(&self) -> &'static str
        where
            E: 'static + pinocchio::program_error::ToStr + TryFrom<u32> {
        match self {
            Self::PdaErr => "Fail to find PDA",
            Self::ReInitPda => "Instruction try to re-init exist PDA",
        }
    }
}

impl From<UniPinoNftErr> for ProgramError {
    fn from(e: UniPinoNftErr) -> Self {
        ProgramError::Custom(e as u32)
    }
}
