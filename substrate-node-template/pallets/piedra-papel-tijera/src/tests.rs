use crate::{mock::*, Error, Event, tipos::CantidadDeJugadores};
use frame_support::{assert_noop, assert_ok};
use frame_system::Origin;
use sp_runtime::traits::Hash;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

use crate::tipos::*;

#[test]
fn registrar_funciona() {

	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(PiedraPapelTijeraModule::registrar(RuntimeOrigin::signed(1)));
		assert_ok!(PiedraPapelTijeraModule::registrar(RuntimeOrigin::signed(2)));

		let jugada_1 = Jugada::Papel;
		let nonce_1: u128 = 719836158792659817324695;
		let concatenacion_1 = jugada_1.using_encoded(|slice_1| nonce_1.using_encoded(|slice_2| [slice_1, slice_2].concat()));
		let hash_1 = <Test as frame_system::Config>::Hashing::hash_of(&concatenacion_1);

        assert_noop!(PiedraPapelTijeraModule::commit(RuntimeOrigin::signed(3), hash_1), Error::<Test>::NoEsJugador);
		assert_ok!(PiedraPapelTijeraModule::commit(RuntimeOrigin::signed(1), hash_1));

		assert_eq!(PiedraPapelTijeraModule::etapa(), Etapa::Commit(CantidadDeJugadores::Uno));

		let jugada_2 = Jugada::Piedra;
		let nonce_2: u128 = 4501394651647051645007136;
		let concatenacion_2 = jugada_2.using_encoded(|slice_1| nonce_2.using_encoded(|slice_2| [slice_1, slice_2].concat()));
		let hash_2 = <Test as frame_system::Config>::Hashing::hash_of(&concatenacion_2);

		assert_ok!(PiedraPapelTijeraModule::commit(RuntimeOrigin::signed(2), hash_2));

		let jugadores = PiedraPapelTijeraModule::jugadores();

		assert_eq!(jugadores.first().unwrap(), &(1, Some(hash_1), None));
		assert_eq!(jugadores.last().unwrap(), &(2, Some(hash_2), None));

		assert_eq!(PiedraPapelTijeraModule::etapa(), Etapa::Reveal(CantidadDeJugadores::Cero));

		assert_noop!(
			PiedraPapelTijeraModule::reveal(RuntimeOrigin::signed(3), Jugada::Papel, 0),
			Error::<Test>::NoEsJugador
		);

		assert_ok!(PiedraPapelTijeraModule::reveal(RuntimeOrigin::signed(1), jugada_1, nonce_1));
		assert_noop!(PiedraPapelTijeraModule::reveal(RuntimeOrigin::signed(2), jugada_1, nonce_1), Error::<Test>::HashIncorrecto);

		assert_ok!(PiedraPapelTijeraModule::reveal(RuntimeOrigin::signed(2), jugada_2, nonce_2));
		assert_eq!(PiedraPapelTijeraModule::etapa(), Etapa::Fin);

		assert_ok!(PiedraPapelTijeraModule::finalizar_juego(RuntimeOrigin::signed(1)));
		System::assert_last_event(Event::Fin { ganador: Some(1) }.into());

	});
}
