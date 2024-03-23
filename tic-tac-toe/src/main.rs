use std::io;
use std::io::Write;

fn check_for_win(game: &[[u8;3]]) -> Option<u8>{
    for row in game{
        let mut count: [i32;2] = [0,0];
        for ch in row{
            if *ch == b'x' { count[0]+=1; }
            if *ch == b'o' { count[1]+=1; }
        }
        if count[0] == 3 { return Some(b'x') }
        else if count[1] == 3 { return Some(b'o') } else { continue; };
    }
    None
}

fn main() {
    println!("Welcome to TIC-TAC-TOE!");
    print!("Enter your name: ");
    let _ = io::stdout().flush();
    let mut name_buf = String::new();
    io::stdin().read_line(&mut name_buf).expect("Error getting the user input");
    println!("Welcome, {}", name_buf.trim());
    let mut game: [[u8; 3]; 3] = [[b'.'; 3]; 3];
    let mut turn = 0;
    loop {
        // display the game field
        print!("\n");
        for row in game{
            for ch in row{
                print!("{} ", ch as char);
            }
            print!("\n");
        }
        print!("\n");
        // get user input
        let mut some_buf = String::new();
        io::stdin().read_line(&mut some_buf).expect("go fuxk urself");
        let parts: Vec<&str> = some_buf.trim().split(" ").collect();
        let r = parts[0].parse::<usize>().unwrap() - 1;
        let c = parts[1].parse::<usize>().unwrap() - 1;
        // update the field
        game[r][c] = match turn % 2 {
            0 => b'x',
            1 => b'o',
            _ => b'.'
        };
        // check for win condition
        match check_for_win(&game) {
            None => print!("\n--- No Winner this round... ---\n"),
            Some(v) => print!("\n--- {} won! ---\n", v as char)
        }
        turn+=1;
    }
}
