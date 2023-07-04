#![cfg_attr(not(feature = "std"), no_std)]

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
pub mod weights;
pub use weights::*;

mod tipos;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::tipos::*;
	use frame_support::pallet_prelude::*;
	use frame_support::sp_runtime::traits::{Hash, AccountIdConversion};
	use frame_support::{PalletId, traits::Currency};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		#[pallet::constant]
		type TokensParaJugar: Get<BalanceDe<Self>>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn jugadores)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Jugadores<T> = StorageValue<_, BoundedVec<Jugador<T>, ConstU32<2>>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn etapa)]
	pub type EtapaDelJuego<T> = StorageValue<_, Etapa, ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// El usuario se registro exitosamente.
		Registrado {quien: CuentaDe<T>},
		/// Jugador comprometido
		Comprometido {quien: CuentaDe<T>, hash: HashDe<T>},
		/// Jugador revelo juego
		Revelado { quien: CuentaDe<T>, jugada: Jugada },
		/// Resultado del juego
		Fin {ganador: Option<CuentaDe<T>>}
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// El usuario ya se registró para un juego, no puede volver a hacerlo.
		YaRegistrado,
		/// El juego está lleno, no puede registrarse.
		JuegoLleno,
		/// La etapa es incorrecta
		EtapaIncorrecta,
		/// Jugador ya comprometido
		YaComprometido,
		/// Participante no es jugador
		NoEsJugador,
		/// El jugador ya revelo su juego
		YaRevelado,
		/// El hash es incorrecto. Verifique nonce o jugada
		HashIncorrecto,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::call_index(0)]
		#[pallet::weight(T::DbWeight::get().writes(1))]
		pub fn registrar(origen: OriginFor<T>) -> DispatchResult {
			// Revisar etapa del juego
			let mut etapa = EtapaDelJuego::<T>::get();
			ensure!(matches!(etapa, Etapa::EsperandoJugadores(_)), Error::<T>::EtapaIncorrecta);
		
			let quien = ensure_signed(origen)?;
			let mut jugadores = Jugadores::<T>::get();
			// Si la etapa es correcta, hay máximo un jugador en el arreglo.
			if let Some(primer_jugador) = jugadores.first() {
				ensure!(primer_jugador.0 != quien, Error::<T>::YaRegistrado);
			}
		
			let jugador = (quien.clone(), None, None); // Jugadores comienzan sin jugada ni compromiso.
			jugadores.force_push(jugador); // Sabemos que no está lleno el arreglo porque la etapa es correcta.
			Jugadores::<T>::set(jugadores);
		
			// Avanzar etapa
			etapa.next();
			EtapaDelJuego::<T>::set(etapa);
		
			Self::deposit_event(Event::Registrado { quien });
		
			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::DbWeight::get().writes(1))]
		pub fn commit(origen: OriginFor<T>, hash: HashDe<T>) -> DispatchResult {
			let mut etapa = EtapaDelJuego::<T>::get();
			ensure!(matches!(etapa, Etapa::Commit(_)), Error::<T>::EtapaIncorrecta);
		
			let quien = ensure_signed(origen)?;
			let mut jugadores = Jugadores::<T>::get();
			let mut encontrado = false;
		  // Hay que tener cuidado con las iteraciones
		  // En este caso no hay problema porque acotamos el arreglo a dos elementos
			for jugador in jugadores.iter_mut() {
				if jugador.0 == quien {
					// Asegurarnos que el jugador no cambie su jugada.
					ensure!(jugador.1 == None, Error::<T>::YaComprometido);
					jugador.1 = Some(hash);
					encontrado = true;
				}
			}
			ensure!(encontrado, Error::<T>::NoEsJugador);
			Jugadores::<T>::set(jugadores);
		
			// Avanzar etapa
			etapa.next();
			EtapaDelJuego::<T>::set(etapa);
		
			Self::deposit_event(Event::Comprometido { quien, hash });
		
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::DbWeight::get().writes(1))]
		pub fn reveal(origen: OriginFor<T>, jugada: Jugada, nonce: u128) -> DispatchResult {
			let mut etapa = EtapaDelJuego::<T>::get();
			ensure!(matches!(etapa, Etapa::Reveal(_)), Error::<T>::EtapaIncorrecta);
		
			let quien = ensure_signed(origen)?;
			let mut jugadores = Jugadores::<T>::get();
			let mut encontrado = false;
			for jugador in jugadores.iter_mut() {
				if jugador.0 == quien {
					// Asegurarnos que el jugador no haya revelado antes.
					ensure!(jugador.2 == None, Error::<T>::YaRevelado);
					let concatenacion = jugada.using_encoded(|slice_1| {
						nonce.using_encoded(|slice_2| [slice_1, slice_2].concat())
					});
					let hash = <T as frame_system::Config>::Hashing::hash_of(&concatenacion);
					ensure!(
						hash == jugador.1.expect("Debe haber un hash en esta etapa"),
						Error::<T>::HashIncorrecto
					);
					jugador.2 = Some(jugada);
					encontrado = true;
				}
			}
			ensure!(encontrado, Error::<T>::NoEsJugador);
			Jugadores::<T>::set(jugadores);
		
			// Avanzar etapa
			etapa.next();
			EtapaDelJuego::<T>::set(etapa);
		
			Self::deposit_event(Event::Revelado { quien, jugada });
		
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::DbWeight::get().writes(1))]
		pub fn finalizar_juego(_origen: OriginFor<T>) -> DispatchResult {
			let etapa = EtapaDelJuego::<T>::get();
			ensure!(etapa == Etapa::Fin, Error::<T>::EtapaIncorrecta);
		
			let jugadores = Jugadores::<T>::get();
			let jugador_1 = jugadores.first().expect("En esta etapa existen los dos jugadores");
			let jugada_1 = jugador_1.2.expect("En esta etapa existen las jugadas");
			let jugador_2 = jugadores.last().expect("En esta etapa existen los dos jugadores");
			let jugada_2 = jugador_2.2.expect("En esta etapa existen las jugadas");
		
			// Lógica para decidir el ganador
			use Jugada::*;
			let ganador = match (jugada_1, jugada_2) {
				(Papel, Piedra) | (Piedra, Tijera) | (Tijera, Papel) => Some(jugador_1.0.clone()),
				(Piedra, Papel) | (Tijera, Piedra) | (Papel, Tijera) => Some(jugador_2.0.clone()),
				_ => None, // Empate
			};
		
			Self::deposit_event(Event::Fin { ganador });
		
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}
