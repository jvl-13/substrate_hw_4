use crate::{mock::*};
use frame_support::{assert_ok};

#[test]
fn test_create_kitty() {
	new_test_ext().execute_with(|| {
		assert_ok!(Kitties::create_kitty(RuntimeOrigin::signed(1), 10));
	});
}

#[test]
fn test_transfer_kitty(){
	new_test_ext().execute_with(||{
		let createkitty = Kitties::create_kitty(RuntimeOrigin::signed(1), 10);
		let get_dna = *Kitties::kitty_owned(1).get(0).unwrap();		
		let transfer_kitty = Kitties::transfer(RuntimeOrigin::signed(1), 2, get_dna);
		assert_ok!(transfer_kitty);
	})
}