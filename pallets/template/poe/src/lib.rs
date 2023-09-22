#![cfg_attr(not(feature = "std"), no_std)]

// pub use pallet::*;

#[frame_support::pallet]
pub mod pallet{
    use frame_support::pallet::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config{
        #[pallet::constant]
        type MaxClaimLength: Get<u32>;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::storage]
    #[pallet::getter(fn proofs)]
    pub type Proofs<T: Config> = StorageMap<
    _, 
    Blake2_128Concat, 
    BoundedVec<u8, T::MaxClaimLength>,
    (T::AccountId, T::BlockNumber)
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
    /// Event emitted when a claim has been created.
    ClaimCreated { who: T::AccountId, claim: T::Hash },
    /// Event emitted when a claim is revoked by the owner.
    ClaimRevoked { who: T::AccountId, claim: T::Hash },
    }

    #[pallet::error]
    pub enum Error<T> {
    /// The claim already exists.
    AlreadyClaimed,
    /// The claim does not exist, so it cannot be revoked.
    NoSuchClaim,
    /// The claim is owned by another account, so caller can't revoke it.
    NotClaimOwner,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
    #[pallet::weight(0)]
    #[pallet::call_index(1)]
    pub fn create_claim(origin: OriginFor<T>, claim: T::Hash) -> DispatchResult {
    // Check that the extrinsic was signed and get the signer.
    // This function will return an error if the extrinsic is not signed.
    let sender = ensure_signed(origin)?;

    // Verify that the specified claim has not already been stored.
    // ensure!(!Claims::<T>::contains_key(&claim), Error::<T>::AlreadyClaimed);
    ensure!(!Proofs::<T>::contains_key(&claim), Error::<T>::ProofAlreadyExist);

    Proofs::<T>::insert(
        &claim,
        (sender.clone(), frame_system::Pallet::<T>::block_number()),
    );

    // // Get the block number from the FRAME System pallet.
    // let current_block = <frame_system::Pallet<T>>::block_number();

    // // Store the claim with the sender and block number.
    // Claims::<T>::insert(&claim, (&sender, current_block));

    // // Emit an event that the claim was created.
    Self::deposit_event(Event::ClaimCreated { sender, claim });

    Ok(().into())
    }

    #[pallet::weight(0)]
    #[pallet::call_index(2)]
    pub fn revoke_claim(
      origin: OriginFor<T>, claim: BoundedVec<u8, T::MaxClaimLength>,
    ) -> DispatchResultWithPostInfo {
    // Check that the extrinsic was signed and get the signer.
    // This function will return an error if the extrinsic is not signed.
    let sender = ensure_signed(origin)?;

    // Get owner of the claim, if none return an error.
    let (owner, _) = Proofs::<T>::get(&claim).ok_or(Error::<T>::ClaimNotExist)?;

    // Verify that sender of the current call is the claim owner.
    ensure!(sender == owner, Error::<T>::NotClaimOwner);

    // Remove claim from storage.
    Proofs::<T>::remove(&claim);

    // Emit an event that the claim was erased.
    Self::deposit_event(Event::ClaimRevoked { sender, claim });
    Ok(().into())
    }
    }
}