use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo, MaxEncodedLen)]
pub enum CantidadDeJugadores {
	Cero,
	Uno,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, TypeInfo, MaxEncodedLen)]
pub enum Etapa {
	EsperandoJugadores(CantidadDeJugadores),
	Commit(CantidadDeJugadores),
	Reveal(CantidadDeJugadores),
	Fin,
}

#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone, Copy, TypeInfo, MaxEncodedLen)]
pub enum Jugada {
	Piedra,
	Papel,
	Tijera,
}

pub type CuentaDe<T> = <T as frame_system::Config>::AccountId;
pub type HashDe<T> = <T as frame_system::Config>::Hash;

pub type Jugador<T> = (CuentaDe<T>, Option<HashDe<T>>, Option<Jugada>);

impl Default for Etapa {
    fn default() -> Self {
        Self::EsperandoJugadores(CantidadDeJugadores::Cero)
    }
}

impl Etapa {
    pub fn next(&mut self) {
        use CantidadDeJugadores::*;
        use Etapa::*;

        *self = match *self {
            EsperandoJugadores(Cero) => EsperandoJugadores(Uno),
            EsperandoJugadores(Uno) => Commit(Cero),
            Commit(Cero) => Commit(Uno),
            Commit(Uno) => Reveal(Cero),
            Reveal(Cero) => Reveal(Uno),
            Reveal(Uno) => Fin,
            Fin => Fin
        };
    }
}