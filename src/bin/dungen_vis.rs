use dungen::room::generate_rooms;
use dungen::vec::vec2i;

// #[cfg(not(tarpaulin))]
fn main() {
    println!("Greetings!");

    let mut rng = rand::rng();
    let rooms = generate_rooms(vec2i(100, 100), Some(15), &mut rng);
    println!("{:#?}", rooms);
}
