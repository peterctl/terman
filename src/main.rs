#![feature(core_intrinsics)]
#![feature(slice_index_methods)]

extern crate vte;

use vte::{
    Parser,
    Perform,
};

struct Performer;

impl Perform for Performer {
    fn hook(
        &mut self,
        params: &[i64],
        intermediates: &[u8],
        ignore: bool,
        action: char,
    ) {
        println!(
            "[hook] params: {:?}, intermediates: {:?}, ignore: {}, action: {}",
            params,
            intermediates,
            ignore,
            action,
        );
    }

    fn unhook(&mut self) {
        println!("[unhook]");
    }

    fn put(&mut self, byte: u8) {
        println!("[put] byte: {}", byte);
    }

    fn print(&mut self, ch: char) {
        println!("[print] char: {}", ch);
    }

    fn execute(&mut self, byte: u8) {
        println!("[execute] byte: {}", byte);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        println!(
            "[esc_dispatch] intermediates: {:?}, ignore: {}, byte: {}",
            intermediates,
            ignore,
            byte,
        );
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        println!(
            "[osc_dispatch] params: {:?}, bell_terminated: {}",
            params,
            bell_terminated,
        );
    }

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, action: char) {
        println!(
            "[csi_dispatch] params: {:?}, intermediates: {:?}, ignore: {}, action: {}",
            params,
            intermediates,
            ignore,
            action,
        );
    }
}

// fn main() {
    // use std::io::Read;
    // let mut parser = Parser::new();
    // let mut performer = Performer{};
    // let mut stdin = std::io::stdin();
//
    // let mut buf: [u8; 1024] = [0; 1024];
//
    // while let Ok(n) = stdin.read(&mut buf) {
        // for byte in &buf[..n] {
            // parser.advance(&mut performer, *byte);
        // }
    // }
// }

mod grid;
mod util;
mod ansi;
use util::P;
use grid::Grid;

fn print_grid(grid: &Grid) {
    // let lines = grid.lines(..).unwrap();
    // for line in lines {
        // print!("|");
        // for cell in line.iter() {
            // print!("{}", cell.ch.unwrap_or(' '));
        // }
        // println!("|");
    // }
}

fn main() {
    let mut grid = Grid::new(P(10, 10));
    // grid.cell_mut(P(5, 5)).unwrap().ch = Some('c');
    // println!("{:?}", grid);
    print_grid(&grid);
}
