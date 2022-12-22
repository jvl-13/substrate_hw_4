#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::pallet_prelude::DispatchResult;
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// use frame_support::pallet_prelude::*;

use frame_system::pallet_prelude::*;
// use sp_std::vec::Vec;
use scale_info::TypeInfo;
pub type Id = u32;
use sp_runtime::ArithmeticError;
use frame_support::{
	pallet_prelude::*,
	traits::{Time, Randomness},
	BoundedVec
};
use frame_support::dispatch::fmt;



#[frame_support::pallet]
pub mod pallet {

	//use frame_system::Config;

// use core::ops::Bound;

pub use super::*;
	#[derive(Clone, Encode, Decode, PartialEq, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Kitty<T: Config> {
		pub dna: T::Hash,
		pub price: u64,
		pub gender: Gender,
		pub owner: T::AccountId,
		pub create_date: MomentOf<T>,
	}

	#[derive(Clone, Encode, Decode, PartialEq, Copy, RuntimeDebug, TypeInfo, MaxEncodedLen)]
	pub enum Gender {
		Male,
		Female,
	}


	impl<T:Config> fmt::Debug for Kitty<T> {
		fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
			f.debug_struct("Kitty")
			 .field("dna", &self.dna)
			 .field("price", &self.price)
			 .field("gender", &self.gender)
			 .field("owner", &self.owner)
			 .field("create_date", &self.create_date)
			 .finish()
		}
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		
		type Time: Time;
		type KittyDNARandom: Randomness<Self::Hash, Self::BlockNumber>;
		
		#[pallet::constant]
		type MaxKittiesOwned: Get<u32>;
	}


	type MomentOf<T> = <<T as Config>::Time as Time>::Moment;



	#[pallet::storage]
    #[pallet::getter(fn kitties_total)]
    pub type KittiesTotal<T: Config> = StorageValue<_, u32, ValueQuery>;


	#[pallet::storage]
	#[pallet::getter(fn kitty_id)]
	pub type KittyId<T> = StorageValue<_, Id, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_kitty)]
	pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitty_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BoundedVec<T::Hash, T::MaxKittiesOwned>, ValueQuery>;



	#[pallet::genesis_config]
    pub struct GenesisConfig<T: Config>{
        pub genesis_kitties: Vec<T::AccountId>
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                genesis_kitties: Vec::new()
            }
        }
    }

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self){
			for item in self.genesis_kitties.iter() {
				let (gen_dna, gen_gender) = Pallet::<T>::gen_dna_gender();

                let kitty = Kitty {
					dna: gen_dna.clone(),
					price : 0u64,
					owner: item.clone(),
					gender: gen_gender,
					create_date: T::Time::now()
				};

				KittiesOwned::<T>::try_mutate(&item.clone(), |list_kitty| {
					list_kitty.try_push(gen_dna.clone())
				}).unwrap();

				Kitties::<T>::insert(gen_dna.clone(), kitty);
			}
		}
	}



	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new kitty was successfully created.
		Created { kitty: T::Hash, owner: T::AccountId },
		Transferred { from: T::AccountId, to: T::AccountId, kitty: T::Hash },

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		DuplicateKitty,
		TooManyOwned,
		NoKitty,
		NotOwner,
		TransferToSelf,
	}

	#[pallet::hooks]
    impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(0)]
		pub fn create_kitty(origin: OriginFor<T>, price: u64) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let owner = ensure_signed(origin)?;

			let (dna, gender) = Self::gen_dna_gender();
			
			let create_date = T::Time::now();

			let kitty = Kitty::<T> { 
				dna: dna.clone(), 
				price , 
				gender, 
				owner: owner.clone(),
				create_date,
			};

			// Check kitty total overflow, if not +1
            let kitty_total = Self::kitties_total();
            let new_kitty_total = kitty_total.checked_add(1).ok_or(ArithmeticError::Overflow)?;


			// Check if the kitty does not already exist in our storage map
			ensure!(!Kitties::<T>::contains_key(&kitty.dna), Error::<T>::DuplicateKitty);

			// Performs this operation first as it may fail
			let current_id = KittyId::<T>::get();
			let next_id = current_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			// Append kitty to KittiesOwned
			// KittiesOwned::<T>::append(&owner, kitty.dna.clone());
			KittiesOwned::<T>::try_mutate(&owner, |list_kitty| {
				list_kitty.try_push(dna.clone())
			}).map_err(|_| <Error<T>>::TooManyOwned)?;

			log::info!("New kitty: {:?}", kitty);

			KittiesTotal::<T>::put(new_kitty_total);
		
			// Write new kitty to storage
			Kitties::<T>::insert(kitty.dna.clone(), kitty);
			KittyId::<T>::put(next_id);

			// Deposit our "Created" event.
			Self::deposit_event(Event::Created { kitty: dna, owner: owner.clone()});

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			dna: T::Hash,
		) -> DispatchResult {
			// Make sure the caller is from a signed origin
			let from = ensure_signed(origin)?;

			let mut kitty = Kitties::<T>::get(&dna).ok_or(Error::<T>::NoKitty)?;
			ensure!(kitty.owner == from, Error::<T>::NotOwner);
			ensure!(from != to, Error::<T>::TransferToSelf);

			let mut from_owned = KittiesOwned::<T>::get(&from);
			// let mut from_owned = Self::kitty_owned(&from);

			// Remove kitty from list of owned kitties.
			if let Some(ind) = from_owned.iter().position(|ids| *ids == dna) {
				from_owned.swap_remove(ind);
			} else {
				return Err(Error::<T>::NoKitty.into());
			}

			KittiesOwned::<T>::try_mutate(&to, |list_kitty| {
				list_kitty.try_push(dna.clone())
			}).map_err(|_| <Error<T>>::TooManyOwned)?; 
			
			log::info!("Transfered kitty to: {:?}", to.clone());

			kitty.owner = to.clone();
            <Kitties<T>>::insert(dna.clone(), kitty);
            <KittiesOwned<T>>::insert(from.clone(), from_owned);


			Self::deposit_event(Event::Transferred { from, to, kitty: dna });

			Ok(())
		}

	}
}

impl<T:Config> Pallet<T> {
	fn gen_dna_gender() -> (T::Hash, Gender) {
		let mut random = T::KittyDNARandom::random(&b"dna"[..]).0;

		while Self::get_kitty(&random) != None {
			random = T::KittyDNARandom::random(&b"dna"[..]).0;
		};

		let gender = if random.encode()[0] % 2 == 0 {
			Gender::Male
		} else {
			Gender::Female
		};

		return (random, gender)
	}
}

