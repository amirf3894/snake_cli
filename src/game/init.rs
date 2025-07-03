use crossterm::{
    cursor::{self, MoveTo},
    event::read,
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
    },
};

use std::{
    io::{self, Stdout, Write, stdout},
    process::exit,
};

use crate::game::snake::add_food;

pub fn start(playground: &mut [[char; 256]; 256]) -> io::Result<Stdout> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let len = playground.len();
    for i in 0..len {
        playground[0][i] = '#';
        playground[i][0] = '#';
        playground[len - 1][i] = '#';
        playground[i][len - 1] = '#';
    }
    (0..200).for_each(|_| add_food(playground));

    Ok(stdout)
}

pub fn end(text: &str, stdout: &mut Stdout) -> io::Result<()> {
    let size = size()?;
    //println!("{}", text);
    for (i, phrase) in text.split("\n").enumerate() {
        execute!(
            stdout,
            MoveTo((size.0 - phrase.len() as u16) / 2, size.1 / 2 + i as u16),
            cursor::Show
        )?;
        // println!("HAHAHAHAH");
        write!(stdout, "{}", phrase)?;
        stdout.flush()?;
    }

    read()?;
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    //exit(0);
    Ok(())
}

pub fn print_wall(stdout: &mut Stdout) -> io::Result<()> {
    let terminal_size = size()?;
    execute!(stdout, MoveTo(0, 0)).unwrap();
    let horizen = "#".repeat(terminal_size.0 as usize);
    write!(stdout, "{horizen}").unwrap();
    (1..=terminal_size.1).for_each(|y| {
        execute!(stdout, MoveTo(0, y)).unwrap();
        write!(stdout, "#").unwrap();
        execute!(stdout, MoveTo(terminal_size.0, y)).unwrap();
        write!(stdout, "#").unwrap();
    });
    execute!(stdout, MoveTo(0, terminal_size.1))?;
    write!(stdout, "{horizen}")?;
    Ok(())
}
