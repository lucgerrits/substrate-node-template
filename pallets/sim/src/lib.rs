#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
	decl_error, decl_event, decl_module, decl_storage, ensure, weights::Pays, StorageMap,
};

use frame_system::ensure_root;
use frame_system::ensure_signed;

use sp_std::vec::Vec;

/// Configure the pallet by specifying the parameters and types on which it depends.
pub trait Config: frame_system::Config {
	/// Because this pallet emits events, it depends on the runtime's definition of an event.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct CrashType<BlockNumber> {
	block_number: BlockNumber,
	data: Vec<u8>
}

// The pallet's runtime storage items.
decl_storage! {
	trait Store for Module<T: Config> as SimModule {
		/// List of factory IDs added by the admin (sudo)
		Factories: map hasher(blake2_128_concat) T::AccountId => T::BlockNumber; //factory => block nb

		/// List of car ID added by the factories
		/// (
		///
		///
		///
		/// )
		Cars: map hasher(blake2_128_concat) T::AccountId => (T::AccountId, T::BlockNumber); //car => factory , block nb

		/// List of car crashes added by a car
		/// Crashes are declared in a map. Each car_id contains a vector of data hashes:
		/// (
		///   car_id => Vec<data_hash>,
		///   car_id => Vec<data_hash>,
		///   ...
		/// )
		Crashes: map hasher(blake2_128_concat) T::AccountId => Vec<CrashType<T::BlockNumber>>;
	}
}

// Pallets use events to inform users when important changes are made.
decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		//, Factory = Vec<u8>, Car = Vec<u8>, Data = Vec<u8>
		/// Event when a factory has been added to storage.
		FactoryStored(AccountId),
		/// Event when a car has been added to storage by a factory.
		CarStored(AccountId, AccountId),
		/// Event when data (hash) is stored by a car and an account.
		CrashStored(AccountId, Vec<u8>),
	}
);

// Errors inform users that something went wrong.
decl_error! {
	pub enum Error for Module<T: Config> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// Factory is not in storage: permission denied.
		UnknownFactory,
		/// Car is not in storage: permission denied
		UnknownCar,
		/// Car is already in storage
		CarAlreadyStored,
		/// Factory is already in storage
		FactoryAlreadyStored,
	}
}

// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// Errors must be initialized if they are used by the pallet.
		type Error = Error<T>;

		// Events must be initialized if they are used by the pallet.
		fn deposit_event() = default;

		/// Dispatchable that takes a singles value as a parameter (factory ID), writes the value to
		/// storage (factories) and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = (0, Pays::No)]
		pub fn store_factory(origin, factory_id: <T as frame_system::Config>::AccountId) {
			ensure_root(origin)?;

			// Verify that the specified factory_id has not already been stored.
			ensure!(!Factories::<T>::contains_key(&factory_id), Error::<T>::FactoryAlreadyStored);

			// Get the block number from the FRAME System module.
			let current_block = <frame_system::Module<T>>::block_number();

			// Store the factory_id with the sender and block number.
			Factories::<T>::insert(&factory_id, current_block);

			// Emit an event.
			Self::deposit_event(RawEvent::FactoryStored(factory_id));
		}

		/// Dispatchable that takes a singles value as a parameter (factory ID), writes the value to
		/// storage (factories) and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = (0, Pays::No)]
		pub fn store_car(origin, car_id: <T as frame_system::Config>::AccountId) {
			let who = ensure_signed(origin)?;

			// Verify that the specified factory_id exists.
			ensure!(Factories::<T>::contains_key(&who), Error::<T>::UnknownFactory);

			// Verify that the specified car_id has not already been stored.
			ensure!(!Cars::<T>::contains_key(&who), Error::<T>::CarAlreadyStored);

			// Get the block number from the FRAME System module.
			let current_block = <frame_system::Module<T>>::block_number();

			// Store the factory_id with the sender and block number.
			Cars::<T>::insert(&car_id, (&car_id, current_block));

			// Emit an event.
			Self::deposit_event(RawEvent::CarStored(car_id, who));
		}

		/// Dispatchable that takes a singles value as a parameter (data hash), writes the value to
		/// storage (crashes) and emits an event. This function must be dispatched by a signed extrinsic.
		#[weight = (0, Pays::No)]
		pub fn store_crash(origin, data_hash: Vec<u8>) {
			let who = ensure_signed(origin)?;

			// Verify that the specified car_id has not already been stored.
			ensure!(Cars::<T>::contains_key(&who), Error::<T>::UnknownCar);

			// Get the block number from the FRAME System module.
			let current_block = <frame_system::Module<T>>::block_number();
			let my_crash = CrashType {
				data: data_hash.clone(),
				block_number: current_block
			};
			if Crashes::<T>::contains_key(&who) {
				//if car has already crashes, we append a new crash
				let mut car_crashes = Crashes::<T>::get(&who);
				car_crashes.push(my_crash);
				
				// Update the crashes by removing + inserting
				// Remove old crahes.
				Crashes::<T>::remove(&who);
				// Store the crash in state.
				Crashes::<T>::insert(&who, car_crashes);
			} else {
				//if car has not crashes, we create a new crash
				let mut new_crash:Vec<CrashType<T::BlockNumber>> = Vec::new();
				new_crash.push(my_crash);

				// Store the crash in state.
				Crashes::<T>::insert(&who, new_crash);
			}

			// Emit an event.
			Self::deposit_event(RawEvent::CrashStored(who, data_hash));
		}

	}
}
