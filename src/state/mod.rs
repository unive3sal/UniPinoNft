macro_rules! impl_state_accessors {
    ($state_ty:ty) => {
        #[allow(dead_code)]
        pub fn try_from_bytes(data: &[u8]) -> Result<&Self, ::pinocchio::error::ProgramError> {
            let state = ::bytemuck::try_from_bytes::<$state_ty>(data)
                .map_err(|_| ::pinocchio::error::ProgramError::InvalidAccountData)?;
            if state.discriminator != Self::DISCRIMINATOR {
                return Err(::pinocchio::error::ProgramError::InvalidAccountData);
            }
            Ok(state)
        }

        #[allow(dead_code)]
        pub fn try_from_bytes_mut(
            data: &mut [u8],
        ) -> Result<&mut Self, ::pinocchio::error::ProgramError> {
            let state = ::bytemuck::try_from_bytes_mut::<$state_ty>(data)
                .map_err(|_| ::pinocchio::error::ProgramError::InvalidAccountData)?;
            if state.discriminator != Self::DISCRIMINATOR {
                return Err(::pinocchio::error::ProgramError::InvalidAccountData);
            }
            Ok(state)
        }
    };
}

pub(crate) use impl_state_accessors;

pub mod nft_meta;
pub mod platform;
pub mod user;
