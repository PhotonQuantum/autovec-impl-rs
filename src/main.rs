use autovec::auto_vec;

fn main() {
    let a = vec![Some(1), Some(4), None];
    let b = vec![Some(2), None, Some(6)];
    println!("{:?}", add_vec(a, b));

    let a = vec![
        (Location { x: 1, y: 2 }, Location { x: 4, y: 9 }, Location { x: 9, y: 7 }),
        (Location { x: 3, y: 2 }, Location { x: 4, y: 1 }, Location { x: 4, y: 7 }),
        (Location { x: 2, y: 2 }, Location { x: 4, y: 2 }, Location { x: 5, y: 7 })
    ];

    println!("{:?}", fn_4_vec(a));
}

// trivial test case
#[auto_vec]
fn add(a: Option<usize>, b: Option<usize>) -> Option<usize> {
    if let Some(a) = a {
        if let Some(b) = b {
            return Some(a + b);
        }
    }
    return None;
}

struct Location {
    x: i64,
    y: i64,
}

// arguments with auto-unpacking can also be handled.
#[auto_vec]
fn fn_4((Location { x, .. }, Location { y, .. }, Location { x: x2, .. }): (Location, Location, Location)) -> i64 {
    x * y * x2
}
