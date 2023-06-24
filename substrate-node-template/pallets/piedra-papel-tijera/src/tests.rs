use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use frame_system::Origin;

#[test]
fn registrar_funciona() {
	new_test_ext().execute_with(|| {
		assert_ok!(PiedraPapelTijeraModule::registrar(RuntimeOrigin::signed(1).into()));
		let jugadores = PiedraPapelTijeraModule::jugadores();
		assert_eq!(jugadores.len(), 1);
		assert_eq!(jugadores.first(), Some(&1));

		assert_noop!(PiedraPapelTijeraModule::registrar(RuntimeOrigin::signed(1).into()), Error::<Test>::YaRegistrado);

		assert_ok!(PiedraPapelTijeraModule::registrar(RuntimeOrigin::signed(2).into()));
		let jugadores = PiedraPapelTijeraModule::jugadores();
		assert_eq!(jugadores.len(), 2);
		assert_eq!(jugadores.first(), Some(&1));
		assert_eq!(jugadores.last(), Some(&2));

		assert_noop!(PiedraPapelTijeraModule::registrar(RuntimeOrigin::signed(3).into()), Error::<Test>::JuegoLleno);
	});
}
